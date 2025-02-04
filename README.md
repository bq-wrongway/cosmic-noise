# Cosmic Noise

Applet for playing background noise, heavily inspired by Blanket

## Galery 

<![dark](https://ibb.co/nSwzHgx)> <![light](https://ibb.co/mVMhMtvg)>


#### Disclaimer:
At some point this will be distributed as flatpak / deb file, untill then best way to install is to download repo and build binary with 


### How to install

```rust
cargo build --release
```
then just copy binary to some location on your computed that is part of the path.

Sadly, this is still not enough, you will need to for now manually copy icons and sounds to required places :


```
 there are 4 icons under resource/icons folder of this repo, they need to go to :
/usr/share/icons/Pop/scalable/actions
(!!! you need to copy icons themself not the folder)

and folder sounds (also found under resources folder of this repo) should be copied either to 
$HOME/.local/share/cosmic-noise
$HOME/.config/cosmic-noise

```

Last but not least, this needs to be run as an applet, so you need to copy 

```
io.github.bq-wrongway.CosmicNoise.desktop
 ```
to 

```
/usr/share/applications/
```

After this you should be good to go, you just need to add applet to your panel, which if everything was done correctly you will be able to do from cosmic settins.

