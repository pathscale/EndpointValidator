
run-honey:
  cargo run -- --services-path ../backend/docs/services.json --config-path ./docs/config.toml
  

run-pays:
  cargo run -- --services-path ../pays.online-backend/docs/services.json --config-path ./docs/config.toml
  
