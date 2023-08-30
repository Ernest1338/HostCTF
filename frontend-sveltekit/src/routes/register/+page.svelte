<script lang="ts">
	import { onMount } from 'svelte';
	onMount(() => {
		document.addEventListener('keydown', (event: KeyboardEvent) => {
			if (event.key == 'Enter') {
				if (location.pathname == '/register') {
					register();
				}
			}
		});
	});

	import { BACKEND_URL } from '../../config';
	import Infobox, { showInfo } from '../../components/infobox.svelte';

	let username = '';
	let email = '';
	let password = '';
	let confirm_password = '';
	let submitting = false;

	async function register() {
		if (submitting) {
			return;
		}
		submitting = true;
		const user = {
			username: username,
			email: email,
			password: password,
			confirm_password: confirm_password
		};
		const response = await fetch(BACKEND_URL + '/register', {
			method: 'POST',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(user)
		});
		const response_json = await response.json();

		if (response_json['status'] == 'OK') {
			username = '';
			email = '';
			password = '';
			confirm_password = '';
			showInfo('success', 'Register successful!');
		} else {
			showInfo('warning', response_json['cause']);
			setTimeout(function () {
				submitting = false;
			}, 1000);
		}
	}
</script>

<Infobox />
<h1>Register</h1>
<form>
	<label for="username">Username</label>
	<input type="text" placeholder="Username" id="username" bind:value={username} /><br />

	<label for="email">E-Mail</label>
	<input type="email" placeholder="example@mail.com" id="email" bind:value={email} /><br />

	<label for="password">Password</label>
	<input type="password" placeholder="Password" id="password" bind:value={password} /><br />

	<label for="confirm_password">Confirm password</label>
	<input type="password" placeholder="Password" id="confirm_password" bind:value={confirm_password} /><br />

	<input type="button" value="Submit" disabled={submitting} on:click={register} />
</form>
