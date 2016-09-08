echo export PATH=\"\$USERPROFILE/.cargo/bin:\$PATH\" > ~/.bash_profile
export PATH="$USERPROFILE/.cargo/bin:$PATH"

which cargo
if [ $? -ne 0 ]; then
	curl https://sh.rustup.rs -sSf | sh
fi

which xargo
if [ $? -ne 0 ]; then
	cargo install xargo
fi

git clone https://github.com/dclews/rocket.git
cd rocket
cargo install
