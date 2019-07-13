.PHONY: clean

lambda-bin: target/x86_64-unknown-linux-musl/release/line-plotter
	docker run --rm -it \
		-v $$PWD:/volume \
		-v cargo-cache:/root/.cargo/registry \
		clux/muslrust:stable \
		cargo build --features aws-lambda --release

lambda: lambda-bin
	cp -nf target/x86_64-unknown-linux-musl/release/line-plotter target/x86_64-unknown-linux-musl/release/bootstrap
	zip -j lambda.zip target/x86_64-unknown-linux-musl/release/bootstrap

clean:
	$(RM) lambda.zip
	

# vim: noexpandtab
