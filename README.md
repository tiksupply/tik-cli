The TIK mainnet genesis is happening on June 10, 2024, at 00:00:00  (UTC). 

# TIK-CLI : Tik Command-line Interface Tool

Tik-cli is a simple command line tool that you can use to mine, check rewards and claim tik.

# Download tik cli

- <a href="https://github.com/tiksupply/tik-cli/releases/download/v1.0.1/tik_windows.zip" target="_blank">Windows</a>

   After decompressing the archive, you will get a file named tik.exe. Open the Windows command line tool. Navigate to the current directory, for example.

         cd c:\tik_windows
    


- <a href="https://github.com/tiksupply/tik-cli/releases/download/v1.0.1/tik_linux.zip" target="_blank">Linux</a>

   After decompressing the archive, you will get a file named tik.


# Creat a keypair

Windows:

      tik.exe gen
       
Linux:

      ./tik gen

- Please note that the keypair in the current directory will look like this:  0xeacafedc18e18848570431fa8ba41ac28bfbca427cb323c105dee4dc32ca13c9.key
- Please deposit at least 1 SUI to start the mining.


# Start mining

Windows:

      tik.exe --keypair <KEYPAIR_FILEPATH> mine

Linux:

      ./tik --keypair <KEYPAIR_FILEPATH> mine

Use the --lock parameter to lock rewards and increase your share. Eg.

      ./tik --keypair <KEYPAIR_FILEPATH> mine --lock 8 

Your share has increased from 10 to 18, and You can only claim your rewards after 8 days. 

# Check rewards

Windows:

       tik.exe --keypair <KEYPAIR_FILEPATH> rewards

Linux:

       ./tik --keypair <KEYPAIR_FILEPATH> rewards

# Claim TIK to your wallet 

Windows:

       tik.exe --keypair <KEYPAIR_FILEPATH> claim

Linux:

       ./tik --keypair <KEYPAIR_FILEPATH> claim

# Show the suiprivate key from a keypair

Windows:

       tik.exe --keypair <KEYPAIR_FILEPATH> prikey

Linux:

       ./tik --keypair <KEYPAIR_FILEPATH> prikey


# Import your suiprivate key to create a keypair

Windows:

       tik.exe Import <suiprivate string> 

Linux:

       ./tik Import <suiprivate string>


# Usage
tik [OPTIONS] --keypair <KEYPAIR_FILEPATH> <COMMAND>

Commands:

       mine     Start mining.
       rewards  Check how much TIK you've earned.
       claim    Claim rewards to your wallet.
       gen      Generate a keypair.
       prikey   Show the suiprivate key from a keypair.
       import   Import your suiprivate key to create a keypair.
       help     Print this message or the help of the given subcommand(s)


Options:

       --rpc <NETWORK_URL>           Network address of your RPC provider default: https://fullnode.testnet.sui.io:443
       --keypair <KEYPAIR_FILEPATH>  Filepath for the keypair to use
       --testnet                     For testnet
       -h, --help                   Print help
       -V, --version                Print version


# Build

- Install RUST. 

Windows: Check this out -->  <a href="https://www.rust-lang.org/tools/install" target="_blank">https://www.rust-lang.org/tools/install</a>

Linux:


      sudo apt update
      sudo apt install build-essential -y
      curl https://sh.rustup.rs -sSf | sh
      . "$HOME/.cargo/env"



- build
         
       git clone https://github.com/tiksupply/tik-cli
       cd tik-cli
       cargo build --release
