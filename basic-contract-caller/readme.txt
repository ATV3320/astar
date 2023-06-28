1. In basic-contract-caller folder:
   For base contract:
   
   cargo contract build --manifest-path other_contract/Cargo.toml
   cargo contract upload --manifest-path other_contract/Cargo.toml --suri //Alice
   
   (note the hash)
   
   For base contract:
   
   cargo contract build
   
   cargo contract instantiate     --constructor new     --args <YOUR HASH>     --suri //Alice --salt $(date +%s) -x
   
   (note the contract address )
   
   cargo contract call --contract <YOUR CONTRACT ADDRESS>     --message flip_and_get --suri //Alice -x



		🤞🏻🤞🏻🤞🏻
