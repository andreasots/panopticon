import {MDCTextField} from '@material/textfield';

let text_fields = document.querySelectorAll(".mdc-text-field");
for (let i = 0; i < text_fields.length; i++) {
    new MDCTextField(text_fields[i]);
}
