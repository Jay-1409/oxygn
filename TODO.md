# Below are some ideas that i find cool , to implement

- Make web visulization using some rust based web library and ultimatly WASM
- Enable the config to be updated in runtime 
    - Probably spawn a tokio process to pooll the config using hashes, and check for a changein hash 
        - Can be computationally heavy  

    - To enable or disable this feature should be the users choice 

    - if you enable this then the healthy check poller might fail, check EDGE CASES in `pool.rs`

    

