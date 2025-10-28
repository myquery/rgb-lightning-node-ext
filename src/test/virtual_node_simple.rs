use crate::virtual_htlc::{VirtualHtlc, VirtualHtlcStatus, RgbTransfer};
use bitcoin::secp256k1::{PublicKey, Secp256k1, SecretKey};
use lightning::ln::{PaymentHash, PaymentPreimage};
use rgb_lib::ContractId;
use std::str::FromStr;

#[test]
fn test_virtual_htlc_api_structure() {
    // Test that the virtual HTLC API compiles and structures work
    let secp = Secp256k1::new();
    let from_key = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[1u8; 32]).unwrap());
    let to_key = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[2u8; 32]).unwrap());
    
    let payment_hash = PaymentHash([42u8; 32]);
    let btc_amount = 1000000; // 1000 sats in msat
    
    // Test RGB transfer
    let rgb_transfer = Some(RgbTransfer {
        contract_id: ContractId::from_str("rgb1qyfe883hey6jrgj2xvk5g3dfmfqfzm7a4wez4pd2krf7ltsxffd6u6nrvjvvnc8vt02v7").unwrap(),
        amount: 100,
    });
    
    // Create virtual HTLC structure
    let virtual_htlc = VirtualHtlc {
        payment_hash,
        from_virtual_node: from_key,
        to_virtual_node: to_key,
        btc_amount_msat: btc_amount,
        rgb_transfer: rgb_transfer.clone(),
        status: VirtualHtlcStatus::Pending,
    };
    
    // Verify structure
    assert_eq!(virtual_htlc.status, VirtualHtlcStatus::Pending);
    assert_eq!(virtual_htlc.btc_amount_msat, 1000000);
    assert!(virtual_htlc.rgb_transfer.is_some());
    
    println!("✅ Virtual HTLC API structure test passed");
}

#[test]
fn test_virtual_htlc_structure() {
    let secp = Secp256k1::new();
    let from_key = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[1u8; 32]).unwrap());
    let to_key = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[2u8; 32]).unwrap());
    
    let virtual_htlc = VirtualHtlc {
        payment_hash: PaymentHash([42u8; 32]),
        from_virtual_node: from_key,
        to_virtual_node: to_key,
        btc_amount_msat: 1000000,
        rgb_transfer: Some(RgbTransfer {
            contract_id: ContractId::from_str("rgb1qyfe883hey6jrgj2xvk5g3dfmfqfzm7a4wez4pd2krf7ltsxffd6u6nrvjvvnc8vt02v7").unwrap(),
            amount: 100,
        }),
        status: VirtualHtlcStatus::Pending,
    };
    
    assert_eq!(virtual_htlc.status, VirtualHtlcStatus::Pending);
    assert_eq!(virtual_htlc.btc_amount_msat, 1000000);
    assert!(virtual_htlc.rgb_transfer.is_some());
}

#[test]
fn test_virtual_settlement_structure() {
    use crate::virtual_htlc::VirtualSettlement;
    
    let secp = Secp256k1::new();
    let from_key = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[1u8; 32]).unwrap());
    let to_key = PublicKey::from_secret_key(&secp, &SecretKey::from_slice(&[2u8; 32]).unwrap());
    
    let settlement = VirtualSettlement {
        payment_hash: PaymentHash([42u8; 32]),
        preimage: PaymentPreimage([1u8; 32]),
        btc_settled: 1000000,
        rgb_settled: Some(RgbTransfer {
            contract_id: ContractId::from_str("rgb1qyfe883hey6jrgj2xvk5g3dfmfqfzm7a4wez4pd2krf7ltsxffd6u6nrvjvvnc8vt02v7").unwrap(),
            amount: 100,
        }),
        from_virtual_node: from_key,
        to_virtual_node: to_key,
    };
    
    assert!(settlement.has_rgb_transfer());
    assert_eq!(settlement.btc_settled, 1000000);
    assert_eq!(settlement.rgb_amount(), Some(100));
}