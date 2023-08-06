<h1><p align=center>HostCTF</p></h1>
<h3><p align=center>🚀 Highly-performant & easy to customize CTF platform written from scratch in 🦀 Rust + Axum</p></h3>
<br><br>

## Compilation and Deployment

Compilation:
```bash
cargo build --release
```

Deployment:
```bash
./target/release/host-ctf
```

By default the server will start on 0.0.0.0 interface and port 3000.

## Customization

To customize CTF looks/branding - modify the template files inside the `templates` directory

To add/modify CTF challenges - modify the `challenges.json` file (you can also put files in the `static` directory for hosting)

## Screenshots

![Main page](https://user-images.githubusercontent.com/45213563/258657575-a51dc554-48a5-4e0b-8e4f-ba87dee08f2b.png)
![Challenges subpage](https://user-images.githubusercontent.com/45213563/258657585-19bdccd4-ab07-42b8-8a27-41e8748d7926.png)
![Scoreboard subpage](https://user-images.githubusercontent.com/45213563/258657592-7a4a4708-a873-40e5-9fe6-391c322e9b1d.png)

## Note

This project is a rewrite of my previous CTF platform - <a href="https://github.com/Ernest1338/LuaCTF">LuaCTF</a> - hence the
nearly exact copy of the frontend.

## License

This project is licensed under the MIT license
