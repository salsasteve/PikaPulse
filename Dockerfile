FROM --platform=$BUILDPLATFORM rust:1.73.0-buster AS rust_fix

# Set up cross-compilation
RUN rustup target add arm-unknown-linux-gnueabihf 

# Enable armhf architecture and install required packages for cross-compilation
RUN dpkg --add-architecture armhf && \
    apt-get update && \
    apt-get -y install gcc-arm-linux-gnueabihf binutils-arm-linux-gnueabihf libasound2-dev:armhf pkg-config

WORKDIR /app
COPY .cargo ./.cargo
COPY Cargo.toml Cargo.lock main.rs ./

# Set up environment variables for cross-compilation
ENV PKG_CONFIG_PATH=/usr/lib/arm-linux-gnueabihf/pkgconfig
ENV PKG_CONFIG_LIBDIR=/usr/lib/arm-linux-gnueabihf
ENV PKG_CONFIG_ALLOW_CROSS=1
ENV CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER=arm-linux-gnueabihf-gcc

RUN cargo build --release --target arm-unknown-linux-gnueabihf

# Move the binary to a location free of the target since that is not available in the next stage.
RUN cp target/arm-unknown-linux-gnueabihf/release/pika_pulse .
