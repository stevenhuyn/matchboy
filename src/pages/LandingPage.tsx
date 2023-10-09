import { createSignal, onCleanup, onMount } from "solid-js";
import init, { connect, send_message, get_history } from "matchlib";

export const LandingPage = () => {
  const [chat, setChat] = createSignal("");
  const [history, setHistory] = createSignal<string[]>([]);

  onMount(async () => {
    let res = await init();
    console.log(res);
    await connect();

    setTimeout(() => {
      console.log("Getting history");
      let h = get_history();
      console.log(h);
      setHistory(h);
    }, 1000);
  });

  const handleInput = (e: Event) => {
    setChat((e.target as HTMLInputElement).value);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && chat()) {
      setHistory((prevHistory) => [...prevHistory, chat()]);
      setChat("");
      send_message(chat());
    }
  };

  onCleanup(() => {
    // cleanup if needed, e.g. clear intervals, listeners, etc.
  });

  return (
    <>
      <h1 class="font-light text-4xl m-6">Match Boy</h1>
      <input
        type="text"
        placeholder="Type here"
        enterkeyhint="send"
        class="input input-bordered w-full max-w-xs"
        value={chat()}
        onInput={handleInput}
        onKeyDown={handleKeyDown}
      />
      <div>
        {history().map((item) => (
          <div>{item}</div>
        ))}
      </div>
    </>
  );
};
