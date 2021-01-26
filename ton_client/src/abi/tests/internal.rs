use std::sync::Arc;

use ton_types::Result;

use crate::abi::internal::{is_empty_pubkey, resolve_pubkey, create_tvc_image};
use crate::abi::{Signer, DeploySet};
use crate::ClientContext;

#[test]
fn test_is_empty_pubkey() -> Result<()> {
    let pubkey = ed25519_dalek::PublicKey::from_bytes(&[0; 32])?;

    assert!(is_empty_pubkey(&pubkey));

    let pubkey = ed25519_dalek::PublicKey::from_bytes(&[1; 32])?;
    assert!(!is_empty_pubkey(&pubkey));

    let mut array = [0; 32];
    array[0] = 1;
    let pubkey = ed25519_dalek::PublicKey::from_bytes(&array)?;
    assert!(!is_empty_pubkey(&pubkey));

    Ok(())
}

#[tokio::test]
async fn test_resolve_pubkey() -> Result<()> {
    let context = Arc::new(ClientContext::new(Default::default())?);

    assert!(resolve_pubkey(&context, &None, &Signer::None ).await?.is_none());

    let external_pub_key = "1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF";
    let external_signer = Signer::External {
        public_key: external_pub_key.to_owned()
    };
    let resolved = resolve_pubkey(&context, &None, &external_signer).await?;
    assert_eq!(resolved, Some(external_pub_key.to_owned()));

    let tvc = base64::encode(include_bytes!("data/SafeMultisigWallet.tvc"));

    let mut deploy_set = DeploySet {
        tvc: tvc.clone(),
        ..Default::default()
    };

    let mut image = create_tvc_image("", None, &tvc)?;
    let resolved = resolve_pubkey(
        &context,
        &Some((&deploy_set, image.clone())),
        &external_signer,
    ).await?;

    assert_eq!(resolved, Some(external_pub_key.to_owned()));

    let tvc_pubkey_empty = ed25519_dalek::PublicKey::from_bytes(&[0; 32])?;
    image.set_public_key(&tvc_pubkey_empty)?;

    let resolved = resolve_pubkey(
        &context,
        &Some((&deploy_set, image.clone())),
        &external_signer,
    ).await?;

    assert_eq!(resolved, Some(external_pub_key.to_owned()));

    let tvc_pubkey_1 = ed25519_dalek::PublicKey::from_bytes(&[1; 32])?;
    image.set_public_key(&tvc_pubkey_1)?;

    let resolved = resolve_pubkey(
        &context,
        &Some((&deploy_set, image.clone())),
        &external_signer,
    ).await?;

    assert_eq!(resolved, Some(hex::encode(tvc_pubkey_1.as_bytes())));

    let initial_pub_key = "1234567890123456789012345678901234567890123456789012345678901234";
    deploy_set.initial_pubkey = Some(initial_pub_key.to_owned());

    let resolved = resolve_pubkey(
        &context,
        &Some((&deploy_set, image.clone())),
        &external_signer,
    ).await?;

    assert_eq!(resolved, Some(initial_pub_key.to_owned()));

    Ok(())
}