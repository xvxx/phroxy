<!--
      /            \    /      
 ___ (___  ___  ___ \  /       
|   )|   )|   )|   ) \/  \   )
|__/ |  / |    |__/  /\   \_/ 
|                   /  \   /  
                   /    \
--> <p align="center"> <img src="./img/logo.png"> <br> 
<a href="https://github.com/dvkt/phroxy/releases">
<img src="https://img.shields.io/crates/v/phroxy">
</a>
</p>


`phroxy` is a small, multi-threaded web server that proxies Gopher
requests through HTTP. It's meant to be run locally and was written
for [gogo](https://github.com/dvkt/gogo), a WebKit Gopher client.

Run it in a terminal then visit the local URL in your favorite web
browser to burrow through the Gophersphere with ease.

## screenies

|![Screenshot](./img/cabin.png)|![Screenshot](./img/sdf.png)|
|:-:|:-:|
| The Lonely Cabin | sdf.org |

|![Screenshot](./img/correct.png)|![Screenshot](./img/gopherproject.png)|
|:-:|:-:|
| gopherproject.org | gopherproject.org |


## usage

    Usage:
    
        phroxy [options]

    Options:

        -p, --port NUM    Port to bind to.
        -h, --host NAME   Hostname to bind to.
    
    Other flags:  
    
        -h, --help        Print this screen.
        -v, --version     Print phd version.

## installation

phroxy is currently only available through https://crates.io/:

    cargo install phroxy

## development

    cargo run -- -p 8080

## credits

phroxy's design is based on 
[phetch](https://github.com/dvkt/phetch)
and inspired by
[Gaufre](https://gitlab.com/commonshost/gaufre).

The proxy idea comes from older gopher/web proxy sites like
https://gopher.floodgap.com/gopher/.

It was made for gogo, which was inspired by lartu's
[OpenNapkin](https://github.com/Lartu/OpenNapkin) client.
