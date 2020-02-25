use anyhow::Result;
use log::info;

use futures::future::try_join_all;

use crate::device::DeviceRegistration;

fn parse_image_url(image_url: &str) -> Result<(String, String)> {
    let registry_index = image_url.find('/').unwrap();
    let registry = image_url[..registry_index].to_string();
    let rest = &image_url[registry_index + 1..];
    let image_index = rest.find('@').unwrap();
    let image = rest[..image_index].to_string();
    Ok((registry, image))
}

pub async fn download_image(image_url: &str, registration: &DeviceRegistration) -> Result<()> {
    let (registry, image) = parse_image_url(image_url)?;
    let username = format!("d_{}", registration.uuid);

    let client = dkregistry::v2::Client::configure()
        .registry(&registry)
        .insecure_registry(false)
        .username(Some(username))
        .password(Some(registration.api_key.clone()))
        .build()
        .unwrap();

    let dclient = authenticate_client(client, &image).await.unwrap();

    info!("Downloading image manifest...");
    let manifest = dclient.get_manifest(&image, "latest").await.unwrap();

    let layers_digests = manifest.layers_digests(None).unwrap();

    info!("Downloading {} layers...", layers_digests.len());
    let blob_futures = layers_digests
        .iter()
        .map(|layer_digest| dclient.get_blob(&image, &layer_digest))
        .collect::<Vec<_>>();

    let blobs = try_join_all(blob_futures).await.unwrap();
    info!("All layers downloaded");

    let path = &format!("{}", &image).replace("/", "_");
    let path = std::path::Path::new(&path);

    std::fs::create_dir(&path).unwrap();
    let can_path = path.canonicalize().unwrap();

    unpack(&blobs, &can_path)?;

    Ok(())
}

pub async fn authenticate_client(
    mut client: dkregistry::v2::Client,
    image: &str,
) -> Result<dkregistry::v2::Client, dkregistry::errors::Error> {
    if !client.is_v2_supported().await? {
        return Err("API v2 not supported".into());
    }

    if client.is_auth(None).await? {
        return Ok(client);
    }

    let login_scope = format!("repository:{}:pull", image);
    let token = client.login(&[&login_scope]).await?;

    if !client.is_auth(Some(token.token())).await? {
        Err("Login failed".into())
    } else {
        info!("Logged in");
        Ok(client.set_token(Some(token.token())).clone())
    }
}

fn unpack(layers: &[Vec<u8>], target_dir: &std::path::Path) -> Result<()> {
    info!("Unpacking layers to {}", target_dir.to_string_lossy());
    for (index, layer) in layers.iter().enumerate() {
        info!("Unpacking layer {}", index + 1);
        let gz_dec = flate2::read::GzDecoder::new(layer.as_slice());
        let mut archive = tar::Archive::new(gz_dec);
        archive.set_preserve_permissions(true);
        archive.set_unpack_xattrs(true);
        for entry in archive.entries()? {
            let mut file = entry?;
            if !clean_whiteouts(&mut file, target_dir)? {
                let _ = file.unpack_in(target_dir);
            }
        }
    }
    info!("All layers unpacked");
    Ok(())
}

fn clean_whiteouts(
    file: &mut tar::Entry<flate2::read::GzDecoder<&[u8]>>,
    target_dir: &std::path::Path,
) -> Result<bool> {
    let path = file.path()?;
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("/"));
    if let Some(fname) = path.file_name() {
        let wh_name = fname.to_string_lossy();
        if wh_name == ".wh..wh..opq" {
            return Ok(true);
        } else if wh_name.starts_with(".wh.") {
            let rel_parent = std::path::PathBuf::from("./".to_string() + &parent.to_string_lossy());

            let real_name = wh_name.trim_start_matches(".wh.");
            let abs_real_path = target_dir.join(&rel_parent).join(real_name);

            if abs_real_path.is_dir() {
                std::fs::remove_dir_all(&abs_real_path)?;
            } else {
                std::fs::remove_file(&abs_real_path)?;
            }

            return Ok(true);
        }
    }

    Ok(false)
}
