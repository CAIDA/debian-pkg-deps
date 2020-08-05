# Debian Package Repository Package Utils

This program extracts the Packages file from the specified OS/Distribution on
the CAIDA Debian package repository, and print out the list of package sources
in a order that follows internal dependency.

```
USAGE:
    debian-pkg-deps --os <os> --distro <distro>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --distro <distro> 
    -o, --os <os>
```

The following commands prints out the list of all packages on the CAIDA package repository for Ubuntu Focal distro:
`debian-pkg-dep --os ubuntu --distro focal`

```
libtimeseries0
avro-c
pytimeseries
libipmeta2
libbgpstream2
libndagserver
pybgpstream
pyipmeta
libbgpview2
```

Note that the packages are ordered in that no internal dependency errors should happen if installing with this order. This is useful when bootstrapping the package repository for a new distro release.
