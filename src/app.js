import "@fortawesome/fontawesome-free/css/all.min.css";

const tabs = [...document.querySelectorAll(".tabs button")];

for (const tab of tabs) {
  tab.addEventListener("click", () => {
    for (const button of tabs) button.classList.remove("active");
    tab.classList.add("active");
  });
}
