# make-first-firefox-profile-default

Does what it says on the tin:
it makes sure that your first Firefox profile is selected as the default one,
even after you launch a different Firefox profile.
This is intended as a workaround for [Bug 1918195][],
and thus hopefully temporary.

Only Linux systems are supported,
because I don’t know the right path to `profiles.ini` on other platforms,
nor would I know how to watch the file for changes there.
The instructions below also assume basic familiarity with the comand line.

## Installation

First of all, make a backup of your `profiles.ini` file.
This program has been tested on exactly one (1) Firefox installation so far,
so there’s certainly a risk that it will break your install.

```sh
cp ~/.mozilla/firefox/profiles.ini ~/.mozilla/firefox/profiles.ini-backup-before-mffpd
```

Whenever you want to restore the backup later, you should run:

```sh
cp ~/.mozilla/firefox/profiles.ini-backup-before-mffpd ~/.mozilla/firefox/profiles.ini
```

Then, assuming you’re on a system that uses [systemd][]
and have already installed [Git][git-install], [Make][make-install] and [Rust][rust-install],
installation should be relatively simple:

```sh
git clone https://github.com/lucaswerkmeister/make-first-firefox-profile-default.git
make -C make-first-firefox-profile-default install
systemctl --user daemon-reload
systemctl --user enable --now make-first-firefox-profile-default.path
```

Now, systemd should watch `profiles.ini` for changes,
and run the program to update the file (changing the default profile back to the first one)
whenever Firefox modifies it.

To uninstall, the following commands should work:

```sh
systemctl --user disable --now make-first-firefox-profile-default.path
make -C make-first-firefox-profile-default uninstall
systemctl --user daemon-reload
```

## License

[Blue Oak Model License 1.0.0](./LICENSE.md).

[Bug 1918195]: https://bugzilla.mozilla.org/show_bug.cgi?id=1918195
[systemd]: https://systemd.io/
[git-install]: https://git-scm.com/downloads/linux
[make-install]: https://www.gnu.org/software/make/
[rust-instal]: https://www.rust-lang.org/tools/install
