.PHONY: check

CARGO = cargo

check:
	$(CARGO) $(CARGOFLAGS) check
	$(CARGO) $(CARGOFLAGS) fmt --check
