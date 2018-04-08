import {MDCDialog} from '@material/dialog';

const dialog = new MDCDialog(document.getElementById("details-dialog")!);

interface MessageDetails {
  [key: string]: string;
}

function display_details(event: Event) {
  event.preventDefault();

  const element = <HTMLElement>event.target;
  const data: MessageDetails = JSON.parse(element.dataset.hit!);

  const desc = document.getElementById("details-dialog-description")!
  while (desc.firstChild) {
    desc.removeChild(desc.firstChild);
  }

  const table = document.createElement("table");
  for (const key in data) {
    if (!data.hasOwnProperty(key)) {
      continue;
    }
    const row = document.createElement("tr");
    table.appendChild(row);

    const name = document.createElement("td");
    name.appendChild(document.createTextNode(key));
    row.appendChild(name);

    const value = document.createElement("td");
    value.appendChild(document.createTextNode(data[key]));
    row.appendChild(value);
  }

  desc.appendChild(table);

  dialog.show();
}

const detail_buttons = document.getElementsByClassName("hit-details");
for (let i = 0; i < detail_buttons.length; i++) {
  detail_buttons[i].addEventListener("click", display_details);
}
