ARG TON_LABS_TYPES_IMAGE=tonlabs/ton-labs-types:latest
ARG TON_LABS_BLOCK_IMAGE=tonlabs/ton-labs-block:latest
ARG TON_LABS_VM_IMAGE=tonlabs/ton-labs-vm:latest
ARG TON_LABS_ABI_IMAGE=tonlabs/ton-labs-abi:latest
ARG TON_EXECUTOR_IMAGE=tonlabs/ton-executor:latest
ARG TON_SDK_IMAGE=tonlabs/ton-sdk:latest

FROM alpine:latest as ton-sdk-src
RUN addgroup --gid 1000 jenkins && \
    adduser -D -G jenkins jenkins
COPY --chown=jenkins:jenkins Cargo.* *.md LICENSE /tonlabs/TON-SDK/
COPY --chown=jenkins:jenkins graphite      /tonlabs/TON-SDK/graphite
COPY --chown=jenkins:jenkins ton_client    /tonlabs/TON-SDK/ton_client
COPY --chown=jenkins:jenkins ton_sdk       /tonlabs/TON-SDK/ton_sdk
COPY --chown=jenkins:jenkins wallet_client /tonlabs/TON-SDK/wallet_client

FROM $TON_LABS_TYPES_IMAGE as ton-labs-types-src
FROM $TON_LABS_BLOCK_IMAGE as ton-labs-block-src
FROM $TON_LABS_VM_IMAGE as ton-labs-vm-src
FROM $TON_LABS_ABI_IMAGE as ton-labs-abi-src
FROM $TON_EXECUTOR_IMAGE as ton-executor-src
FROM $TON_SDK_IMAGE as ton-sdk-source

FROM alpine:latest as ton-sdk-full
RUN addgroup --gid 1000 jenkins && \
    adduser -D -G jenkins jenkins && \
    apk update && apk add zip
COPY --from=ton-labs-types-src --chown=jenkins:jenkins /tonlabs/ton-labs-types /tonlabs/ton-labs-types
COPY --from=ton-labs-block-src --chown=jenkins:jenkins /tonlabs/ton-labs-block /tonlabs/ton-labs-block
COPY --from=ton-labs-vm-src    --chown=jenkins:jenkins /tonlabs/ton-labs-vm    /tonlabs/ton-labs-vm
COPY --from=ton-labs-abi-src   --chown=jenkins:jenkins /tonlabs/ton-labs-abi   /tonlabs/ton-labs-abi
COPY --from=ton-executor-src   --chown=jenkins:jenkins /tonlabs/ton-executor   /tonlabs/ton-executor
COPY --from=ton-sdk-source     --chown=jenkins:jenkins /tonlabs/TON-SDK        /tonlabs/TON-SDK
WORKDIR /tonlabs/ton-executor
RUN sed -e "s/\/tonlabs\/ton-block/\/tonlabs\/ton-labs-block/g" Cargo.toml | \
    sed -e "s/\/tonlabs\/ton-types/\/tonlabs\/ton-labs-types/g" | \
    sed -e "s/\/tonlabs\/ton-vm/\/tonlabs\/ton-labs-vm/g" > tmp.toml && \
    rm Cargo.toml && mv tmp.toml Cargo.toml

FROM rust:latest as ton-sdk-build
RUN apt -qqy update && apt -qyy install apt-utils && \
    curl -sL https://deb.nodesource.com/setup_12.x | bash - && \
    apt-get install -qqy nodejs
COPY --from=ton-sdk-full /tonlabs /tonlabs
WORKDIR /tonlabs/TON-SDK/ton_client/client
RUN node build.js