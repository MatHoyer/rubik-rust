NAME := rubik
SRC  := Cargo.toml Cargo.lock $(wildcard */Cargo.toml) $(shell find engine cli -name "*.rs")

all: $(NAME)

$(NAME): $(SRC)
	cargo build --release --locked -p cli
	cp target/release/$(NAME) $(NAME)

clean:
	cargo clean

fclean: clean
	rm -rf $(NAME) wasm/pkg web/dist web/node_modules

re: fclean all

test:
	cargo test --locked

bench: $(NAME)
	cargo run --release --locked -p engine --example bench -- --states 10000 --cli ./$(NAME) --runs 100
	cargo run --release --locked -p engine --example bench -- --algo kociemba --states 200 --cli ./$(NAME) --runs 20

# Bonus: browser package in wasm/pkg (needs the wasm32-unknown-unknown target and wasm-pack).
wasm:
	wasm-pack build wasm --release --target web --out-dir pkg

wasm-test:
	wasm-pack test --node --release wasm

# Bonus: 3D web interface in web/dist (needs Node). Serve it over HTTP, e.g. make serve.
web: wasm
	cd web && npm ci && npm test && npm run build

serve: web
	python3 -m http.server -d web/dist 8080

.PHONY: all clean fclean re test bench wasm wasm-test web serve
