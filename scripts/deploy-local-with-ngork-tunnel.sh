#!/bin/bash

dfx deploy --no-wallet

backend_canister_id=$(dfx canister id backend)
icx_proxy_port=$(dfx info webserver-port)

ngrok http http://$backend_canister_id.localhost:$icx_proxy_port/ --request-header-add="host: $backend_canister_id.localhost:$icx_proxy_port"
