.PHONY: check

CARGO = cargo

INSTALL = install
INSTALL_PROGRAM = $(INSTALL)
INSTALL_DATA = $(INSTALL) -m 644

all: target/release/make-first-firefox-profile-default

target/release/make-first-firefox-profile-default:
	$(CARGO) $(CARGOFLAGS) build --release

check:
	$(CARGO) $(CARGOFLAGS) check
	$(CARGO) $(CARGOFLAGS) test
	$(CARGO) $(CARGOFLAGS) fmt --check
	$(CARGO) $(CARGOFLAGS) clippy

install: target/release/make-first-firefox-profile-default make-first-firefox-profile-default.service make-first-firefox-profile-default.path
	$(INSTALL_PROGRAM) $< ~/.local/lib/
	$(INSTALL_DATA) make-first-firefox-profile-default.service make-first-firefox-profile-default.path ~/.local/share/systemd/user/

clean:
	$(CARGO) $(CARGOFLAGS) clean

uninstall:
	$(RM) ~/.local/lib/make-first-firefox-profile-default ~/.local/share/systemd/user/make-first-firefox-profile-default.service ~/.local/share/systemd/user/make-first-firefox-profile-default.path
