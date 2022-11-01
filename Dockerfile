FROM rustlang/rust:nightly as builder
COPY . .
RUN cargo install --path .


FROM debian
COPY --from=builder /usr/local/cargo/bin/multisub /usr/local/bin/multisub
ENTRYPOINT ["multisub"]
