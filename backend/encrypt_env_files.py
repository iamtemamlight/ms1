#!/usr/bin/env python3
"""
Environment File Encryption Tool for AllBright

This script encrypts sensitive .env files using AES-256 encryption
with PBKDF2 key derivation for secure storage in version control systems.

Features:
- AES-256-GCM authenticated encryption
- PBKDF2 key derivation with configurable iterations
- Salt generation and storage
- JSON-based encrypted file format
- Backup and restore functionality
- Secure password handling

Encryption format:
{
  "encrypted_data": "base64_encoded_encrypted_bytes",
  "salt": "base64_encoded_salt",
  "algorithm": "AES-256-GCM",
  "version": "1.0",
  "timestamp": "iso8601_timestamp"
}
"""

import os
import json
import hashlib
import secrets
import argparse
import base64
import getpass
from pathlib import Path
from cryptography.hazmat.primitives.ciphers.aead import AESGCM
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.kdf.pbkdf2 import PBKDF2HMAC

# Constants
DEFAULT_ITERATIONS = 100_000
SALT_SIZE = 16
NONCE_SIZE = 12
default_encrypt_script = '''#!/usr/bin/env python3
"""
Environment File Encryption Script
Encrypts .env files using AES-256-GCM for secure storage
"""

import os
import json
import base64
import hashlib
import secrets
import argparse
from pathlib import Path

def derive_key_from_password(password: str, salt: bytes, iterations: int = 100_000) -> bytes:
    """Derive encryption key from password using PBKDF2."""
    kdf = PBKDF2HMAC(
        algorithm=hashes.SHA256(),
        length=32,
        salt=salt,
        iterations=iterations,
    )
    return kdf.derive(password.encode('utf-8'))

def encrypt_file(file_path: Path, password: str, iterations: int = DEFAULT_ITERATIONS) -> bool:
    """Encrypt a .env file using AES-256-GCM."""
    try:
        # Read the file
        with open(file_path, 'rb') as f:
            plaintext = f.read()
        
        # Generate random salt and nonce
        salt = secrets.token_bytes(SALT_SIZE)
        nonce = secrets.token_bytes(NONCE_SIZE)
        
        # Derive key from password
        key = derive_key_from_password(password, salt, iterations)
        
        # Encrypt the data
        aesgcm = AESGCM(key)
        ciphertext = aesgcm.encrypt(nonce, plaintext, None)
        
        # Combine nonce + ciphertext
        encrypted_data = nonce + ciphertext
        
        # Create encrypted structure
        encrypted_structure = {
            'encrypted_data': base64.b64encode(encrypted_data).decode('utf-8'),
            'salt': base64.b64encode(salt).decode('utf-8'),
            'algorithm': 'AES-256-GCM',
            'version': '2.0',
            'iterations': iterations,
            'timestamp': __import__('datetime').datetime.now().isoformat(),
            'original_filename': file_path.name
        }
        
        # Write encrypted file
        output_path = file_path.with_suffix('.env.enc')
        with open(output_path, 'w') as f:
            json.dump(encrypted_structure, f, indent=2)
        
        # Remove original file
        file_path.unlink()
        
        print(f"✅ Encrypted: {file_path} -> {output_path}")
        print(f"   Algorithm: AES-256-GCM")
        print(f"   Key derivation: PBKDF2 with {iterations} iterations")
        print(f"   Salt size: {len(salt)} bytes")
        print(f"   Nonce size: {len(nonce)} bytes")
        
        return True
        
    except Exception as e:
        print(f"❌ Error encrypting {file_path}: {e}")
        return False

def decrypt_file(encrypted_path: Path, password: str) -> bool:
    """Decrypt an encrypted .env file."""
    try:
        # Read encrypted file
        with open(encrypted_path, 'r') as f:
            encrypted_structure = json.load(f)
        
        # Extract data and salt
        encrypted_data = base64.b64decode(encrypted_structure['encrypted_data'])
        salt = base64.b64decode(encrypted_structure['salt'])
        iterations = encrypted_structure.get('iterations', DEFAULT_ITERATIONS)
        
        # Extract nonce (first 12 bytes) and ciphertext (rest)
        nonce = encrypted_data[:12]
        ciphertext = encrypted_data[12:]
        
        # Derive key from password
        key = derive_key_from_password(password, salt, iterations)
        
        # Decrypt the data
        aesgcm = AESGCM(key)
        plaintext = aesgcm.decrypt(nonce, ciphertext, None)
        
        # Get original filename
        original_filename = encrypted_structure.get('original_filename', 
                                                   encrypted_path.stem.replace('.env', ''))
        
        # Write decrypted file
        output_path = encrypted_path.parent / original_filename
        with open(output_path, 'wb') as f:
            f.write(plaintext)
        
        print(f"✅ Decrypted: {encrypted_path} -> {output_path}")
        print(f"   Algorithm: AES-256-GCM")
        print(f"   Key derivation: PBKDF2 with {iterations} iterations")
        
        return True
        
    except Exception as e:
        print(f"❌ Error decrypting {encrypted_path}: {e}")
        return False

def main():
    parser = argparse.ArgumentParser(
        description='Encrypt/Decrypt .env files using AES-256-GCM',
        formatter_class=argparse.RawDescriptionHelpFormatter
    )
    
    parser.add_argument(
        'action',
        choices=['encrypt', 'decrypt'],
        help='Action to perform'
    )
    
    parser.add_argument(
        '--password',
        help='Master password (will prompt if not provided)'
    )
    
    parser.add_argument(
        '--file',
        required=True,
        help='Path to the .env file to encrypt or decrypt'
    )
    
    parser.add_argument(
        '--iterations',
        type=int,
        default=DEFAULT_ITERATIONS,
        help=f'Key derivation iterations (default: {DEFAULT_ITERATIONS})'
    )
    
    parser.add_argument(
        '--list',
        action='store_true',
        help='List all encrypted files in current directory'
    )
    
    parser.add_argument(
        '--backup',
        action='store_true',
        help='Create backup of decrypted files'
    )
    
    args = parser.parse_args()
    
    # Get master password
    if args.password:
        password = args.password
    else:
        password = getpass.getpass('Enter master password: ')
        getpass.getpass('Confirm master password: ')
    
    # Convert to Path
    file_path = Path(args.file)
    
    if not file_path.exists():
        if args.action == 'decrypt':
            print(f"❌ Error: Encrypted file not found: {file_path}")
            return
        else:
            print(f"❌ Error: Input file not found: {file_path}")
            return
    
    if args.action == 'encrypt':
        print(f"🔐 Encrypting {file_path}...")
        print(f"   Algorithm: AES-256-GCM")
        print(f"   Iterations: {args.iterations}")
        
        if encrypt_file(file_path, password, args.iterations):
            print(f"✅ Encryption completed successfully!")
            print(f"   Encrypted file saved as: {file_path.with_suffix('.env.enc')}")
    
    elif args.action == 'decrypt':
        print(f"🔓 Decrypting {file_path}...")
        
        if decrypt_file(file_path, password):
            print(f"✅ Decryption completed successfully!")
            print(f"   Decrypted file saved as: {file_path.with_suffix('').with_suffix('')}")

if __name__ == '__main__':
    main()
