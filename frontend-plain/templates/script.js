async function getData(url) {
    const response = await fetch(url);

    return response.json()
}

async function genId() {
    return Math.random().toString(36).replace(/[^a-z]+/g, '').substr(2, 10);
}

async function showInfo(type, msg) {
    let icon, style;
    if (type == "success") {
        icon = "✅";
        style = "background-color: #1d4d1d; padding: 0; border: 2px solid #007000;";
    } else if (type == "warning") {
        icon = "⚠️";
        style = "background-color: #b39c2a; color: #ffffff; padding: 0; border: 2px solid #ffb300;";
    } else { return; }

    const id = await genId();
    const cur_endpoint = window.location.pathname;
    if (cur_endpoint == "/challenges" || cur_endpoint == "/" || cur_endpoint == "/logout" || cur_endpoint == "/profile") {
        document.getElementById('banner-box').insertAdjacentHTML('beforebegin', '<article id="' + id + '" style="'
            + style + '"><p style="text-align: center;">' + icon + ' ' + msg + '</p></article>');
    } else if (cur_endpoint == "/register" || cur_endpoint == "/login") {
        document.getElementById('main').insertAdjacentHTML('afterbegin', '<article id="' + id + '" style="'
            + style + '"><p style="text-align: center;">' + icon + ' ' + msg + '</p></article>');
    }
    document.getElementById(id).scrollIntoView(false);

    setTimeout(function() {
        document.getElementById(id).remove();
    }, 15000);
}

function setCookie(name, value) {
    document.cookie = name + "=" + value + ";";
}

function setCookieArray(name, array) {
    document.cookie = name + "=" + JSON.stringify(array) + ";";
}

function appendCookie(name, value) {
    document.cookie = name + "=" + getCookie(name) + value + ";";
}

function appendCookieArray(name, value) {
    let array = getCookieArray(name);
    array.push(value);
    document.cookie = name + "=" + JSON.stringify(array) + ";";
}

// returns true if the value was present already, false otherwise
function appendCookieArrayDistinct(name, value) {
    let array = getCookieArray(name);
    if (!array.includes(value)) {
        array.push(value);
        document.cookie = name + "=" + JSON.stringify(array) + ";";
        return true;
    }
    return false;
}

function getCookieArray(cname) {
    let name = cname + "=";
    let decoded_cookie = decodeURIComponent(document.cookie);
    let ca = decoded_cookie.split(';');
    for (let i=0; i<ca.length; i++) {
        let c = ca[i];
        while (c.charAt(0) == ' ') {
            c = c.substring(1);
        }
        if (c.indexOf(name) == 0) {
            return JSON.parse(c.substring(name.length, c.length));
        }
    }
    return JSON.parse("[]");
}

function getCookie(cname) {
    let name = cname + "=";
    let decoded_cookie = decodeURIComponent(document.cookie);
    let ca = decoded_cookie.split(';');
    for (let i=0; i<ca.length; i++) {
        let c = ca[i];
        while (c.charAt(0) == ' ') {
            c = c.substring(1);
        }
        if (c.indexOf(name) == 0) {
            return c.substring(name.length, c.length);
        }
    }
    return "";
}

function markAsSolved(id) {
    // update cookie
    appendCookieArrayDistinct("solved_chals", id);
    // update visuals
    document.getElementById("chal_" + id).style = "background-color: #1d4d1d;";
}

async function submitFlag(id) {
    console.log("Submitting flag...");

    document.getElementById('submit_' + id).disabled = true;

    const username = getCookie('logged_as');
    const auth_key = getCookie('auth_key');
    const flag = document.getElementById('flag_' + id).value
    const submition = {
        username: username,
        auth_key: auth_key,
        challenge_id: id,
        flag: flag
    };

    const response = await fetch('{{ backend_addr }}/flag_submit', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(submition),
    });
    const response_json = await response.json();

    if (response_json["status"] == "OK") {
        console.log("Flag correct");
        showInfo("success", "Flag accepted!");
        markAsSolved(id);
    } else {
        const cause = response_json["cause"];
        console.log("Flag submit failed: " + cause);
        showInfo("warning", cause);
        setTimeout(function() {
            document.getElementById('submit_' + id).disabled = false;
        }, 1000);
    }
}

async function register() {
    console.log("Registering");

    const user = {
        'username': document.getElementById('username').value,
        'email': document.getElementById('email').value,
        'password': document.getElementById('password').value,
        'confirm_password': document.getElementById('confirm_password').value,
    };

    const response = await fetch('{{ backend_addr }}/register', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(user),
    });
    const response_json = await response.json();

    if (response_json["status"] == "OK") {
        console.log("Register successful");
        document.getElementById('submit').remove();
        document.getElementById('username').value = "";
        document.getElementById('email').value = "";
        document.getElementById('password').value = "";
        document.getElementById('confirm_password').value = "";
        showInfo("success", "Register successful!");
    } else {
        const cause = response_json["cause"];
        console.log("Register failed: " + cause);
        document.getElementById('submit').disabled = true;
        showInfo("warning", cause);
        setTimeout(function() {
            document.getElementById('submit').disabled = false;
        }, 1000);
    }
}

async function login() {
    console.log("Logging in");
    const user = {
        'username': document.getElementById('username').value,
        'password': document.getElementById('password').value,
    };

    const response = await fetch('{{ backend_addr }}/login', {
        method: 'POST',
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        },
        body: JSON.stringify(user),
    });
    const response_json = await response.json();

    if (response_json["status"] == "OK") {
        console.log("Login successful");
        document.getElementById('submit').remove();
        document.getElementById('username').value = "";
        document.getElementById('password').value = "";
        showInfo("success", "Login successful!");
        // recreate solved_chals cookie
        setCookieArray("solved_chals", response_json["solved_chals"]);
        // set auth cookie (bearer from axum, tower auth?)
        setCookie("auth_key", response_json["auth_key"]);
        // set "logged_as" cookie
        setCookie("logged_as", user["username"]);
        handleMenu();
    } else {
        const cause = response_json["cause"];
        console.log("Login failed: " + cause);
        document.getElementById('submit').disabled = true;
        showInfo("warning", cause);
        setTimeout(function() {
            document.getElementById('submit').disabled = false;
        }, 1000);
    }
}

async function handleMenu() {
    var but1 = document.getElementById("menu_additional1");
    var but2 = document.getElementById("menu_additional2");
    if (document.cookie.includes("logged_as")) {
        but1.textContent = "Profile";
        but2.textContent = "Logout";
        but1.href = "/profile";
        but2.href = "/logout";
    } else {
        but1.textContent = "Register";
        but2.textContent = "Login";
        but1.href = "/register";
        but2.href = "/login";
    }
}

async function main() {
    handleMenu();

    const cur_endpoint = window.location.pathname;

    if (cur_endpoint == "/challenges") {

        const challenges = await getData('{{ backend_addr }}/challenges');

        if (challenges.length == undefined) {
            document.getElementById('banner').innerHTML = 'CTF hasn\'t started yet!';
            return;
        }

        var main = document.getElementById('main');
        var solved_chals = getCookieArray("solved_chals");

        for (var cat_id = 0; cat_id < challenges.length; cat_id++) {
            var category = document.createElement('h3');
            category.textContent = challenges[cat_id]['name'];

            main.appendChild(category);

            for (var chal_id = 0; chal_id < challenges[cat_id]['challenges'].length; chal_id++) {
                const challenge = challenges[cat_id]['challenges'][chal_id];
                var challenge_obj = document.createElement('details');

                challenge_obj.id = "chal_" + challenge["id"];

                if (solved_chals.includes(challenge["id"])) {
                    challenge_obj.style = "background-color: #1d4d1d;";
                }

                challenge_obj.insertAdjacentHTML('beforeend', '<summary>' + challenge["name"]
                    + ' - <em style="color:var(--accent);">' + challenge["points"] + '</em></summary>');

                challenge_obj.insertAdjacentHTML('beforeend', '<p>' + challenge["description"] + '</p>');

                if (challenge["hint"] != undefined) {
                    challenge_obj.insertAdjacentHTML('beforeend', '<details><summary>Hint</summary><p>'
                        + challenge["hint"] + '</p></details>');
                }

                if (solved_chals.includes(challenge["id"])) {
                    challenge_obj.insertAdjacentHTML('beforeend', '<form>'
                        + '<input type="text" id="flag_' + challenge["id"] + '" placeholder="flag{...}">'
                        + '<input type="button" id="submit_' + challenge["id"] + '" name="submit" value="Submit" disabled></form>');
                } else {
                    challenge_obj.insertAdjacentHTML('beforeend', '<form>'
                        + '<input type="text" id="flag_' + challenge["id"] + '" placeholder="flag{...}">'
                        + '<input type="button" id="submit_' + challenge["id"] + '" name="submit" value="Submit" onclick="submitFlag('
                        + challenge["id"] + ');"></form>');
                }

                main.appendChild(challenge_obj);

                document.getElementById('flag_' + challenge["id"]).addEventListener('keypress', event => {
                    if (event.keyCode === 13) {
                        event.preventDefault();
                        document.getElementById('submit_' + challenge["id"]).click();
                    }
                });
            }
        }
    } else if (cur_endpoint == "/scoreboard") {
        const sb_data = await getData('{{ backend_addr }}/scoreboard');

        if (sb_data.length == 0) {
            document.getElementById('banner').innerHTML = 'No users yet!';
            return;
        }

        var scoreboard = document.getElementById('scoreboard');

        // TODO: render only X users (+add paging) for better performance
        for (var user_id = 0; user_id < sb_data.length; user_id++) {
            var entry = document.createElement('tr');

            entry.insertAdjacentHTML('beforeend', '<td>' + (user_id + 1) + '</td><td>' + sb_data[user_id]["username"]
                + '</td><td>' + sb_data[user_id]["score"] + '</td>');

            scoreboard.appendChild(entry);
        }
    } else if (cur_endpoint == "/logout") {
        console.log("Logging out");
        const cookies = document.cookie.split(';');

        for (let i=0; i<cookies.length; i++) {
            const cookie = cookies[i];
            const eqPos = cookie.indexOf("=");
            const name = eqPos > -1 ? cookie.substr(0, eqPos) : cookie;
            document.cookie = name + "=;expires=Thu, 01 Jan 1970 00:00:00 GMT";
        }

        showInfo("success", "Logged out");
        handleMenu();
    } else if (cur_endpoint == "/profile") {
        const username = getCookie('logged_as');
        if (username == '') {
            document.getElementById('banner').textContent = "You need to be logged in!";
            return;
        }
        const response = await fetch('{{ backend_addr }}/profile', {
            method: 'POST',
            headers: {
                'Accept': 'application/json',
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ username: username }),
        });
        const data = await response.json();
        if (data["status"] == "OK") {
            const box = document.getElementById('banner-box');
            box.insertAdjacentHTML('afterend', '<p>Score: ' + data["score"] + '</p>');
            box.insertAdjacentHTML('afterend', '<h2>' + username + '</h2>');
        } else {
            showInfo("warning", data["cause"]);
        }
    }
}

main()

