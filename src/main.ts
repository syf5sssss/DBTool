import { createApp } from 'vue'
import App from './App.vue'
import PrimeVue from "primevue/config";
import Aura from '@primevue/themes/aura';
import 'primeflex/primeflex.css';
import 'primeicons/primeicons.css';
import ToastService from 'primevue/toastservice';
import ConfirmationService from 'primevue/confirmationservice';


const app = createApp(App);
app.use(ToastService);
app.use(ConfirmationService);

app.use(PrimeVue, {
    theme: {
        preset: Aura
    }
});
app.mount('#app');