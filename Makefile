NAME := rubik
SRC  := Cargo.toml Cargo.lock $(wildcard */Cargo.toml) $(shell find engine cli -name "*.rs")

all: $(NAME)

$(NAME): $(SRC)
	cargo build --release --locked -p cli
	cp target/release/$(NAME) $(NAME)

clean:
	cargo clean

fclean: clean
	rm -f $(NAME)

re: fclean all

test:
	cargo test --locked

bench: $(NAME)
	cargo run --release --locked -p engine --example bench -- --states 10000 --cli ./$(NAME) --runs 100

.PHONY: all clean fclean re test bench
