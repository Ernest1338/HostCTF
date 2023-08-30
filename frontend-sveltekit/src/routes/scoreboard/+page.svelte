<script lang="ts">
	import { getData } from '$lib';
	import { BACKEND_URL } from '../../config';

	let promise = getData(BACKEND_URL + '/scoreboard');
</script>

<article>
	<h2 style="text-align: center;">Scoreboard</h2>
</article>
{#await promise}
	<!--<p>Loading scoreboard...</p>-->
{:then users}
	{#if users.length == 0}
		<article style="background-color: var(--base);">
			<h3 style="text-align: center;">No users yet.</h3>
		</article>
	{:else}
		<table style="display: revert; width: 100%;">
			<thead>
				<tr>
					<th>Rank</th>
					<th>Username</th>
					<th>Score</th>
				</tr>
			</thead>
			<tbody>
				{#each users as { username, score }, index}
					<tr>
						<td>{index + 1}</td>
						<td>{username}</td>
						<td>{score}</td>
					</tr>
				{/each}
			</tbody>
		</table>
	{/if}
{:catch error}
	<p>{error}</p>
{/await}
