<!--
      /            \    /      
 ___ (___  ___  ___ \  /       
|   )|   )|   )|   ) \/  \   )
|__/ |  / |    |__/  /\   \_/ 
|                   /  \   /  
                   /    \
--> <p align="center"> <img src="./img/logo.png"> <br> 
<a href="https://crates.io/crates/phroxy">
<img src="https://img.shields.io/crates/v/phroxy">
</a>
</p>


`phroxy` is a small, multi-threaded web server that proxies Gopher
requests through HTTP. It's meant to be run locally or behind an HTTPS
proxy and was written for [gogo](https://github.com/xvxx/gogo), a
WebKit-based Gopher desktop client.

To use it locally, run `phroxy` in a terminal then visit the local URL
in your favorite web browser. You'll be burrowin through the
Gophersphere with ease in no time!

If you want to setup a private instance of phroxy on the real web so
you can browse Gopher using your tablet or TV, we recommend running it
behind an HTTPS proxy like [Caddy](https://caddyserver.com/v1/):

    $ cat Caddyfile
    your-website.com
    proxy / localhost:8080
    $ phroxy -p 8080
    Listening at http://0.0.0.0:8080...

## screenies

|![Screenshot](./img/cabin.png)|![Screenshot](./img/sdf.png)|
|:-:|:-:|
| The Lonely Cabin | sdf.org |

|![Screenshot](./img/correct.png)|![Screenshot](./img/gopherproject.png)|
|:-:|:-:|
| gopherproject.org | gopherproject.org |


## usage

        phroxy [options]

    Options:

        -p, --port NUM    Port to bind to.
        -h, --host NAME   Hostname to bind to.
        -g, --gopher URL  Default Gopher URL to load.
    
    Other flags:  
    
        -h, --help        Print this screen.
        -v, --version     Print phroxy version.

## installation

phroxy is currently only available through https://crates.io/:

    cargo install phroxy

## development

    cargo run -- -p 8080

You can set the start screen to your own Gopher server. [phd][phd]
perhaps?

    # start gopher
    phd
    ┬ Listening on 0.0.0.0:7070 at /Users/randy/Code/phroxy

    # then start phroxy
    cargo run -- -p 8080 -g 0.0.0.0:7070
    # and visit it in your web browser
    open http://127.0.0.1:8080

## credits

phroxy's design is based on 
[phetch](https://github.com/xvxx/phetch)
and inspired by
[Gaufre](https://gitlab.com/commonshost/gaufre).

The proxy idea comes from older gopher/web proxy sites like
https://gopher.floodgap.com/gopher/.

It was made for gogo, which was inspired by lartu's
[OpenNapkin](https://github.com/Lartu/OpenNapkin) client.

## todo 

- [ ] return 500s vs 404s accurately
- [ ] user supplied css
- [ ] systemd example
- [ ] man page 
