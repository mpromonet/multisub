FROM rust as builder
COPY . .
RUN cargo install --path .


FROM debian
COPY --from=builder /usr/local/cargo/bin/multisub /usr/local/bin/multisub
CMD ["multisub"]
