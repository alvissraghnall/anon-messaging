
<script lang="ts">
  const { id, username, onClose, onStartMessaging } = $props<{
    id: string,
    username: string,
    onClose: () => void,
    onStartMessaging: () => void
  }>();
</script>

<div tabindex="-3" role="dialog" class="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" onclick={onClose}>
  <div data-testid="inner" class="bg-white p-6 rounded-lg shadow-lg max-w-md w-full" onclick={ev => { 
      ev.stopPropagation();
    }}>
    <h2 class="text-xl font-bold mb-4 text-center">Identity Created</h2>
    <p class="mb-2">Hi, @<span class="font-semibold">{username}</span>, and welcome to Piree!</p>

    <div class="flex flex-col space-y-3">
      <button
        class="bg-green-600 hover:bg-green-700 text-white py-2 px-4 rounded transition"
        onclick={onStartMessaging}>
        Click here to start receiving and sending messages
      </button>

      <button
        class="bg-gray-200 hover:bg-gray-300 text-gray-800 py-2 px-4 rounded transition"
        onclick={onClose}>
        Close
      </button>
    </div>
  </div>
</div>
