import {MDCTextField} from '@material/textfield';
import {MDCSnackbar} from '@material/snackbar';
import {MDCTopAppBar} from '@material/top-app-bar';
import {MDCTemporaryDrawer} from '@material/drawer';
import {MDCSelect} from '@material/select';

const text_fields = document.querySelectorAll(".mdc-text-field");
for (let i = 0; i < text_fields.length; i++) {
  MDCTextField.attachTo(text_fields[i]);
}

const select_fields = document.querySelectorAll(".mdc-select");
for (let i = 0; i < select_fields.length; i++) {
  MDCSelect.attachTo(select_fields[i]);
}

const snackbar_element = document.querySelector(".mdc-snackbar")!;
const snackbar = new MDCSnackbar(snackbar_element);
function show_next_flash() {
  let flashes = <Array<string>>(<any>window).FLASHES;
  const message = flashes.shift();
  if (message != undefined) {
    snackbar.show(<any>{ // The type definition is incorrect, only "message" is required.
      message: message,
    });
  }
};
snackbar_element.addEventListener("MDCSnackbar:hide", show_next_flash);
show_next_flash();

const top_bar_element = document.querySelector(".mdc-top-app-bar")!;
const top_bar = new MDCTopAppBar(top_bar_element);
const drawer = new MDCTemporaryDrawer(document.querySelector("#main-drawer")!);
top_bar_element.addEventListener("MDCTopAppBar:nav", (event) => drawer.open = true);
