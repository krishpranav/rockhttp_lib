const socket = new WebSocket("ws://" + window.location.hostname + ":8129");
socket.addEventListener('open', function(event)  { console.log("Reloading enabled!"); });
socket.addEventListener('message', function(evenet) { location.reload(); });