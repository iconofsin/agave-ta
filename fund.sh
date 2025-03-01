solana-keygen new --force --no-bip39-passphrase -o alice.json &> alice.log
solana-keygen new --force --no-bip39-passphrase -o bob.json &> bob.log

solana --url http://127.0.0.1:8899 airdrop 200 $(solana-keygen pubkey alice.json)
solana --url http://127.0.0.1:8899 airdrop 200 $(solana-keygen pubkey bob.json)

# solana --url http://127.0.0.1:8899 airdrop 200 71A3Q8no9bsbATTvdcTKfAdQtzq81W1PEabdELVNdEfi
# solana --url http://127.0.0.1:8899 airdrop 200 D4MPrZoQKhV158SmmfaHSKBtZF1xCr4q5wn3ScA1dUjM

solana --url http://127.0.0.1:8899 transfer --from ./alice.json $(solana-keygen pubkey bob.json) 10