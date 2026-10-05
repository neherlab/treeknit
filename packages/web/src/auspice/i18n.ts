import sidebar from "auspice/src/locales/en/sidebar.json";
import translation from "auspice/src/locales/en/translation.json";
import i18next from "i18next";

const TREEKNIT_TRANSLATION = {
  "Showing {{x}} of {{y}} genomes": "Showing {{x}} of {{y}} leaves",
  "Showing {{x}} of {{y}} genomes sampled between {{from}} and {{to}}":
    "Showing {{x}} of {{y}} leaves sampled between {{from}} and {{to}}",
  "Click on tip to display more info": "Click to show the details of this leaf",
  "Shift + Click to display more info": "Shift + click to show the details of this branch",
};

export const AUSPICE_I18N = i18next.createInstance({
  resources: { en: { sidebar, translation: { ...translation, ...TREEKNIT_TRANSLATION } } },
  lng: "en",
  fallbackLng: "en",
  defaultNS: "translation",
  interpolation: { escapeValue: false },
  initImmediate: false,
});

await AUSPICE_I18N.init();
