# TON SDK Core Library 1.0


# Dynamic Library Interface


- `tc_create_context` – creates core context
- `tc_request_context_async` – performs async request to core function
- `tc_destroy_context` – closes core context
    
    
# Async JSON Interface (API)

   
- `tc_request_context_async (request_id, method, params_json, on_response)` – sends 
    async request to core
- `on_response (request_id, result_json, error_json)` – receives async response 
    from core
    

# Async App Callbacks (new)


- `context.register_callback` – registers async app defined callback into core  
- `on_callback` – receives callback invocation from core 
- `context.subscribe` – example of callback usage  
- `context.unregister_callback` – unregisters callback


# Core Modules (reorganized)


- `client` – core interoperation functions  
- `crypto` – cryptography
- `abi` – message encoding / decoding according to ABI specs
- `boc` – BOC serialization / manipulation / block structures parsing
- `tvm` – local TVM runtime utilization (get methods, local run, etc.)
- `net` – blockchain interaction functions


# BOC Module


- `parse_account`
- `parse_message`
- `parse_transaction`
- `parse_block`


# ABI Module


- `encode_message` – instead of a lot of encode functions
- `decode_message` – instead of a lot of decode functions  

    
# Net Module


- `process_message` – observable single function with callback support
    instead of a lot of runs deploys, waits for transactions etc.


# Internal Code Refactorings


- `modularity` – each module placed inside own folder
- `similarity` – each API function has parameters represented as a structure 
    and a result represented as a structure
- `encodings` – each var len byte buffer encoded with `base64` 
    and each fixed len byte buffer encoded with `hex`
- `errors` – each module has own errors definitions (instead of huge common file)
- `tests` – each module has own test suite (instead of huge common test suite)
- `self-documented` – each API function and structure has self documented 
    derived at runtime documentation
- `rust-direct` – all underlying API functions are accessible directly 
    from rust applications




