<script lang="ts">
	import { toasts, toast } from '$lib/stores/toast';
</script>

{#if $toasts.length > 0}
	<div class="toast-container">
		{#each $toasts as t (t.id)}
			<div class="toast toast--{t.type}" role="alert">
				<span class="toast__icon">
					{#if t.type === 'success'}✓
					{:else if t.type === 'error'}✕
					{:else if t.type === 'warning'}⚠
					{:else}ℹ
					{/if}
				</span>
				<span class="toast__message">{t.message}</span>
				<button class="toast__close" onclick={() => toast.dismiss(t.id)}>×</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.toast-container {
		position: fixed;
		bottom: 48px;
		right: 16px;
		z-index: 9999;
		display: flex;
		flex-direction: column-reverse;
		gap: 8px;
		max-width: 400px;
		pointer-events: none;
	}
	.toast {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: 6px;
		font-size: 13px;
		line-height: 1.4;
		box-shadow: 0 4px 12px rgba(0,0,0,0.3);
		pointer-events: auto;
		animation: slideIn 0.2s ease-out;
	}
	.toast--info {
		background: var(--color-surface, #2d2d2d);
		color: var(--color-text, #e0e0e0);
		border: 1px solid var(--color-border, #3c3c3c);
	}
	.toast--success {
		background: #1a3a2a;
		color: #4ade80;
		border: 1px solid #2d5a3d;
	}
	.toast--error {
		background: #3a1a1a;
		color: #f87171;
		border: 1px solid #5a2d2d;
	}
	.toast--warning {
		background: #3a2a1a;
		color: #fbbf24;
		border: 1px solid #5a3d1a;
	}
	.toast__icon {
		flex-shrink: 0;
		font-size: 14px;
	}
	.toast__message {
		flex: 1;
		word-break: break-word;
	}
	.toast__close {
		flex-shrink: 0;
		background: none;
		border: none;
		color: inherit;
		opacity: 0.5;
		cursor: pointer;
		font-size: 16px;
		padding: 0 2px;
	}
	.toast__close:hover { opacity: 1; }
	@keyframes slideIn {
		from { opacity: 0; transform: translateY(8px); }
		to { opacity: 1; transform: translateY(0); }
	}
</style>
