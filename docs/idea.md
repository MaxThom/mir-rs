# Idea

1. Config struct printing with secret
    - the debug print, would replace with *
2. Load protobuff
    - lib-dizer has a compiled protobuff to add overhead over the user compiled protobuff.
    - at runtime, lib-dizer add the user protobuff to its schema

3. Device Buffer
    - could have a buffer that capture the message from RabbtMq
    - so we can delay add properties handler to after join_fleet and possibly other features
