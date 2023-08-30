<script lang="ts" context="module">
	import { writable } from 'svelte/store';

	let timeoutId: ReturnType<typeof setTimeout>;

	const alertStore = writable({ show: false, type: '', message: '' });

	export function showInfo(type: string, message: string) {
		if (type == 'warning') {
			message = '⚠️ ' + message;
		} else if (type == 'success') {
			message = '✅ ' + message;
		}
		alertStore.set({ show: true, type, message });
		clearTimeout(timeoutId);
		timeoutId = setTimeout(() => alertStore.set({ show: false, type: '', message: '' }), 5000);
		setTimeout(() => {
			const alertElement = document.getElementById('alert-box');
			if (alertElement) {
				alertElement.scrollIntoView(false);
			}
		}, 10);
	}
</script>

<article id="alert-box" hidden={!$alertStore.show} class={`alert ${$alertStore.type}`}>
	<p>{$alertStore.message}</p>
</article>

<style>
	.alert {
		padding: 0;
		border-radius: 8px;
		text-align: center;
	}

	.success {
		background-color: #1b281b;
		border: 3px solid #007000;
	}

	.warning {
		/* background-color: #b39c2a; */
		color: #ffffff;
		border: 3px solid #ffb300;
	}
</style>
