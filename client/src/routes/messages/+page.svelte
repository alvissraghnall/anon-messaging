<script lang="ts">
  import type { PageProps } from './$types';
  let { data }: PageProps = $props();

  let newMessage = $state('');
  let activeTab = $state('conversation'); // or 'details'

  async function sendMessage() {
    if (!newMessage.trim()) return;

    // Optimistic UI update
    data.messages = [
      ...data.messages,
      {
        id: Date.now().toString(),
        content: newMessage,
        sender: 'current-user-id',
        timestamp: new Date().toISOString(),
        status: 'sent'
      }
    ];

    newMessage = '';
  }
</script>

<div class="flex h-screen bg-gray-50">
  <!-- Sidebar with conversation list -->
  <div class="w-80 border-r border-gray-200 bg-white">
    <div class="p-4 border-b border-gray-200">
      <h2 class="text-xl font-semibold text-gray-800">Messages</h2>
    </div>
    <div class="divide-y divide-gray-200">
      {#each data.conversations as conversation}
        <a
          href={`/messages/${conversation.id}`}
          class="block p-4 hover:bg-gray-100 {data.id === conversation.id ? 'bg-blue-50' : ''}"
        >
          <div class="flex justify-between items-start">
            <div>
              <p class="font-medium text-gray-900">
                {conversation.participants.find(p => p.id !== data.currentUserId)?.name || 'Unknown User'}
              </p>
              <p class="text-sm text-gray-500 truncate max-w-xs">
                {conversation.lastMessage?.content || 'No messages yet'}
              </p>
            </div>
            <span class="text-xs text-gray-400">
              {conversation.lastMessage?.timestamp ? new Date(conversation.lastMessage.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) : ''}
            </span>
          </div>
        </a>
      {/each}
    </div>
  </div>

  <!-- Main chat area -->
  <div class="flex-1 flex flex-col">
    <!-- Chat header -->
    <div class="p-4 border-b border-gray-200 bg-white flex justify-between items-center">
      <h2 class="text-lg font-semibold text-gray-800">
        {data.conversation.participants.find(p => p.id !== data.currentUserId)?.name || 'Conversation'}
      </h2>
      <div class="flex space-x-2">
        <button
          class="p-2 rounded-full hover:bg-gray-100"
          class:bg-gray-100={activeTab === 'details'}
          on:click={() => activeTab = 'details'}
        >
          <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 text-gray-500" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zm-7-4a1 1 0 11-2 0 1 1 0 012 0zM9 9a1 1 0 000 2v3a1 1 0 001 1h1a1 1 0 100-2h-1V9z" clip-rule="evenodd" />
          </svg>
        </button>
      </div>
    </div>

    {#if activeTab === 'conversation'}
      <!-- Messages container -->
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        {#each data.messages as message}
          <div class="flex {message.sender === data.currentUserId ? 'justify-end' : 'justify-start'}">
            <div
              class={`max-w-xs lg:max-w-md px-4 py-2 rounded-lg ${
                message.sender === data.currentUserId
                  ? 'bg-blue-500 text-white rounded-br-none'
                  : 'bg-gray-200 text-gray-800 rounded-bl-none'
              }`}
            >
              <p>{message.content}</p>
              <p class="text-xs mt-1 opacity-70 text-right">
                {new Date(message.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                {message.status === 'read' && ' ✓✓'}
                {message.status === 'delivered' && ' ✓'}
              </p>
            </div>
          </div>
        {/each}
      </div>

      <!-- Message input -->
      <div class="p-4 border-t border-gray-200 bg-white">
        <form on:submit|preventDefault={sendMessage} class="flex space-x-2">
          <input
            type="text"
            bind:value={newMessage}
            placeholder="Type a message..."
            class="flex-1 border border-gray-300 rounded-full px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
          <button
            type="submit"
            class="bg-blue-500 hover:bg-blue-600 text-white rounded-full p-2 w-10 h-10 flex items-center justify-center"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
              <path fill-rule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-8.707l-3-3a1 1 0 00-1.414 0l-3 3a1 1 0 001.414 1.414L9 9.414V13a1 1 0 102 0V9.414l1.293 1.293a1 1 0 001.414-1.414z" clip-rule="evenodd" />
            </svg>
          </button>
        </form>
      </div>
    {:else}
      <!-- Conversation details panel -->
      <div class="flex-1 p-4 overflow-y-auto">
        <div class="bg-white rounded-lg shadow-sm p-4">
          <h3 class="font-medium text-gray-900 mb-2">Participants</h3>
          <ul class="space-y-2">
            {#each data.conversation.participants as participant}
              <li class="flex items-center space-x-2">
                <div class="h-8 w-8 rounded-full bg-gray-300 flex items-center justify-center text-gray-600">
                  {participant.name.charAt(0)}
                </div>
                <span>{participant.name}</span>
              </li>
            {/each}
          </ul>
        </div>
      </div>
    {/if}
  </div>
</div>
