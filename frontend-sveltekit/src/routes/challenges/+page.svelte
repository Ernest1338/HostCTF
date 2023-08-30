<script lang="ts">
	import { appendCookieArrayDistinct, getCookie, getCookieArray, getData } from '$lib';
	import Infobox, { showInfo } from '../../components/infobox.svelte';
	import { BACKEND_URL } from '../../config';

	let promise = getData(BACKEND_URL + '/challenges');
	let submitting = false;
	let solvedChals = getCookieArray('solved_chals');

	async function markAsSolved(id: number) {
		appendCookieArrayDistinct('solved_chals', id);
		setTimeout(function () {
			let button = document.getElementById('button_' + id) as HTMLInputElement;
			if (button) {
				button.disabled = true;
			}
			let details = document.getElementById('details_' + id) as HTMLDetailsElement;
			if (details) {
				details.style.backgroundColor = '#1d4d1d';
				details.open = false;
			}
		}, 0);
	}

	async function submitFlag(id: number, flag: string) {
		if (submitting) {
			return;
		}
		submitting = true;

		const username = getCookie('logged_as');
		const auth_key = getCookie('auth_key');
		const submition = {
			username: username,
			auth_key: auth_key,
			challenge_id: id,
			flag: flag || ''
		};

		const response = await fetch(BACKEND_URL + '/flag_submit', {
			method: 'POST',
			headers: {
				Accept: 'application/json',
				'Content-Type': 'application/json'
			},
			body: JSON.stringify(submition)
		});
		const response_json = await response.json();

		if (response_json['status'] == 'OK') {
			showInfo('success', 'Flag accepted!');
			submitting = false;
			markAsSolved(id);
		} else {
			showInfo('warning', response_json['cause']);
			setTimeout(function () {
				submitting = false;
			}, 1000);
		}
	}
</script>

<Infobox />
<article>
	<h2 style="text-align: center;">Challenges</h2>
</article>
{#await promise}
	<!--<p>Loading challenges...</p>-->
{:then chall_cats}
	{#if chall_cats.length == undefined}
		<article style="background-color: var(--base);">
			<h3 style="text-align: center;">CTF didn't started yet.</h3>
		</article>
	{:else}
		{#each chall_cats as chall_cat}
			<h3>{chall_cat.name}</h3>
			{#each chall_cat.challenges as chall}
				<details id="details_{chall.id}" data-solved={solvedChals.includes(chall.id)}>
					<summary>{chall.name} - <em style="color:var(--accent);">{chall.points}</em></summary>
					<p>
						{chall.description}
						{#if chall.hint}
							<details>
								<summary>Hint</summary>
								<p>{chall.hint}</p>
							</details>
						{/if}
					</p>
					<form on:submit|preventDefault={() => submitFlag(chall.id, chall.flag)}>
						<input type="text" bind:value={chall.flag} placeholder={`flag{...}`} />
						<input
							type="button"
							id="button_{chall.id}"
							on:click={() => submitFlag(chall.id, chall.flag)}
							disabled={submitting || solvedChals.includes(chall.id)}
							value="Submit"
						/>
					</form>
				</details>
			{/each}
		{/each}
	{/if}
{:catch error}
	<p>{error}</p>
{/await}

<style>
	details[data-solved='true'] {
		background-color: #1d4d1d;
	}
</style>
