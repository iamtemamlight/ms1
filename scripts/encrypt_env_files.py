import os
import json
import base64
import hashlib
import secrets
from pathlib import Path
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC
import argparse

def derive_key_from_password(password: str, salt: bytes, iterations: int = 100_000) -> bytes:
    """Derive encryption key from password using PBKDF2."""
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=32,
        salt=salt,
        iterations=iterations,
    )
    return kdf.derive(password.encode('utf-8'))

def encrypt_file(file_path: Path, key: bytes, output_path: Path = None) -> bool:
    """Encrypt a file using AES-256-GCM."""
    try:
        # Read the file content
        with open(file_path, 'rb') as f:
            plaintext = f.read()
        
        # Generate random nonce (12 bytes for GCM)
        nonce = secrets.token_bytes(12)
        
        # Encrypt the data
        aesgcm = AESGCM(key)
        ciphertext = aesgcm.encrypt(nonce, plaintext, None)
        
        # Combine nonce + ciphertext for storage
        encrypted_data = nonce + ciphertext
        
        # Store as base64 JSON
        encrypted_json = {
            "encrypted_data": base64.b64encode(encrypted_data).decode('utf-8'),
            "algorithm": "AES-256-GCM",
            "version": "1.0",
            "timestamp": str(__import__('datetime').datetime.now().isoformat())
        }
        
        # Write encrypted file
        target_path = output_path or file_path.with_suffix('.env.enc')
        with open(target_path, 'w') as f:
            json.dump(encrypted_json, f, indent=2)
        
        print(f"Encrypted: {file_path} -> {target_path}")
        return True
        
    except Exception as e:
        print(f"Failed to encrypt {file_path}: {e}")
        return False

def decrypt_file(encrypted_path: Path, key: bytes, output_path: Path = None) -> bool:
    """Decrypt an encrypted file."""
    try:
        # Read encrypted JSON
        with open(encrypted_path, 'r') as f:
            encrypted_json = json.load(f)
        
        encrypted_data = base64.b64decode(encrypted_json['encrypted_data'])
        
        # Extract nonce and ciphertext
        nonce = encrypted_data[:12]
        ciphertext = encrypted_data[12:]
        
        # Decrypt the data
        aesgcm = AESGCM(key)
        plaintext = aesgcm.decrypt(nonce, ciphertext, None)
        
        # Write decrypted file
        target_path = output_path or encrypted_path.with_suffix('').with_suffix('')
        with open(target_path, 'wb') as f:
            f.write(plaintext)
        
        print(f"Decrypted: {encrypted_path} -> {target_path}")
        return True
        
    except Exception as e:
        print(f"Failed to decrypt {encrypted_path}: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(description='Encrypt/decrypt .env files')
    parser.add_argument('action', choices=['encrypt', 'decrypt'], help='Action to perform')
    parser.add_argument('--password', help='Master password (will prompt if not provided)')
    parser.add_argument('--output-dir', default='.', help='Output directory')
    
    args = parser.parse_args()
    
    workspace_root = Path(__file__).parent
    
    # List of files to encrypt/decrypt
    env_files = [
        workspace_root / '.env',
        workspace_root / '.env.example',
        workspace_root / '.env.bak',
        workspace_root / '.env.production',
        workspace_root / 'backend' / '.env',
        workspace_root / 'backend' / '.env.example',
        workspace_root / 'apps' / 'dashboard' / '.env',
        workspace_root / 'apps' / 'dashboard' / '.env.example',
    ]
    
    # Filter files that exist
    existing_files = [f for f in env_files if f.exists()]
    
    if not existing_files:
        print("No environment files found to process")
        return
    
    print(f"Found {len(existing_files)} environment files to process")
    for f in existing_files:
        print(f"  - {f.relative_to(workspace_root)}")
    
    # Get master password
    password = args.password or input("Enter master password for encryption: ")
    confirm_password = input("Confirm master password: ")
    
    if password != confirm_password:
        print("Passwords don't match. Try again.")
        return
    
    if len(password) < 12:
        print("Password should be at least 12 characters.")
        return
    
    # Generate salt for key derivation
    salt = secrets.token_bytes(16)
    key = derive_key_from_password(password, salt)
    
    if args.action == 'encrypt':
        print(f"\nEncrypting {len(existing_files)} files...")
        success_count = 0
        
        for file_path in existing_files:
            output_dir = Path(args.output_dir)
            output_dir.mkdir(exist_ok=True)
            output_path = output_dir / f"{file_path.name}.enc"
            
            if encrypt_file(file_path, key, output_path):
                success_count += 1
        
        # Save salt to a file
        salt_file = Path(args.output_dir) / 'encryption_salt.dat'
        with open(salt_file, 'wb') as f:
            f.write(salt)
        print(f"Salt saved to: {salt_file}")
        
        print(f"\n✅ Successfully encrypted {success_count}/{len(existing_files)} files")
        print("\nNext steps:")
        print("1. Keep your master password safe")
        print(f"2. Store the salt file securely: {salt_file}")
        print("3. Delete or move original .env files to .env.bak")
        
    elif args.action == 'decrypt':
        print(f"\nDecrypting files...")
        
        # Load salt from salt file
        salt_file = Path(args.output_dir) / 'encryption_salt.dat'
        if not salt_file.exists():
            print(f"Salt file not found: {salt_file}")
            return
        
        with open(salt_file, 'rb') as f:
            salt = f.read()
        
        key = derive_key_from_password(password, salt)
        
        # Find encrypted files
        encrypted_files = list(Path(args.output_dir).glob('*.env.enc'))
        
        if not encrypted_files:
            print("No encrypted files found")
            return
        
        success_count = 0
        for enc_file in encrypted_files:
            output_path = enc_file.with_suffix('')
            if decrypt_file(enc_file, key, output_path):
                success_count += 1
        
        print(f"\n✅ Successfully decrypted {success_count}/{len(encrypted_files)} files")

if __name__ == '__main__':
    main()
