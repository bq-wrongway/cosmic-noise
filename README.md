# Cosmic Noise

Applet for playing background noise, heavily inspired by Blanket


* This apples was intended to be distributed as package/ flatpak
* This package is also very much WIP so basically error handling is not really good, so in order to get this to work you will need to copy resources required on your own

### How to 
```
 clone project to your computer
 then enter project and use
 * cargo build --release
 newly created bin file then you can just copy anywere that you want, that is on path
 io.github.bq-wrongway.CosmicNoise.desktop file you need to copy to 
 /usr/share/application 
 you need to copy icons from resource folder to /usr/share/icons/Cosmic/scalable/apps
and you need to copy sounds directory to $HOME/.local/share/cosmic-noise/


```

Anyways this is now quite convoluted, but i will try to add proper error and fallbacks as soon as i can, i am just kinda quite busy atm.