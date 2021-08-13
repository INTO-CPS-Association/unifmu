# https://hub.docker.com/_/rust?tab=description&page=1&ordering=last_updated
# https://wapl.es/rust/2019/02/17/rust-cross-compile-linux-to-macos.html
# https://john-millikin.com/notes-on-cross-compiling-rust
# https://github.com/phracker/MacOSX-SDKs
# https://github.com/tpoechtrager/apple-libtapi
# https://github.com/multiarch/crossbuild
# https://github.com/dockcross/dockcross
git clone https://github.com/tpoechtrager/osxcross
cd osxcross
wget https://github.com/phracker/MacOSX-SDKs/releases/download/11.3/MacOSX11.3.sdk.tar.xz
mv MacOSX11.3.sdk.tar.xz tarballs/
UNATTENDED=yes OSX_VERSION_MIN=10.7 ./build.sh


