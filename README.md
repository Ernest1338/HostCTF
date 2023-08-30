<h1><p align=center>HostCTF</p></h1>
<h3><p align=center>🚀 Highly-performant & customizable CTF platform written from scratch in 🦀 Rust + Axum</p></h3>
<br><br>

## Compilation and Deployment

If you want to run everything from a `single machine`, you can go inside of the "single-binary-host" directory and

Compile & Deploy: `cargo run --release`

By default the app will start on 0.0.0.0 interface and port 3000.

Another option for deployment is a separate backend and frontend. Here you can mix and match backends and frontends withing this repository (the API is consistent between those).
- backend-fast - Compile & Deploy: `cargo run --release`
- frontend-plain - Compile & Deploy: `cargo run --release`
- frontend-sveltekit - Deploy on a hosting provider like Vercel or Compile & Deploy: `npm run build && npm run preview`

## Customization

To customize CTF looks/branding - modify the template files inside the `templates` directory in either: `single-binary-host/templates` or `frontend-plain/templates`. Or modify the `frontend-sveltekit/src/app.html` file.

To add/modify CTF challenges - modify the `challenges.json` file in either `single-binary-host/challenges.json` or `backend-fast/challenges.json` (you can also put files in the `static` directory for hosting when using frontend-sveltekit or single-binary-host)

## Screenshots

![Main page](https://user-images.githubusercontent.com/45213563/258657575-a51dc554-48a5-4e0b-8e4f-ba87dee08f2b.png)
![Challenges subpage](https://user-images.githubusercontent.com/45213563/258657585-19bdccd4-ab07-42b8-8a27-41e8748d7926.png)
![Scoreboard subpage](https://user-images.githubusercontent.com/45213563/258657592-7a4a4708-a873-40e5-9fe6-391c322e9b1d.png)

## Directory structure
- single-binary-host - Contains a all-in-one HostCTF implementation which does server-side rendering from a single binary
- backend-fast - Contains the "fast" (based on a custom in-memory, commit on write) version of the HostCTF backend
- frontend-plain - Contains a plain html/css/js version of the HostCTF frontend (connects to a HostCTF backend)
- frontend-sveltekit - Contains a sveltekit rewrite of the plain HostCTF frontend (connects to a HostCTF backend)

## Note

This project is a rewrite of my previous CTF platform - <a href="https://github.com/Ernest1338/LuaCTF">LuaCTF</a> - hence the
nearly exact copy of the frontend.

## License

This project is licensed under the MIT license

