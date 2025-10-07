# Maintainer: denkijin <denkijin@proton.me>

pkgname=kcc
pkgver=0.1.0
pkgrel=1
pkgdesc="Kernel Config Checker - fast and flexible kernel config flags checker"
arch=('x86_64')
url="https://github.com/dandenkijin/kcc"
license=('MIT')
depends=('rust')
makedepends=('git')
source=("$pkgname-$pkgver.tar.gz::https://github.com/dandenkijin/kcc/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('SKIP') # Replace with actual sha256 checksum after first build

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
}
