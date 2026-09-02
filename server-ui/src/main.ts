import {createApp} from 'vue'
import App from './App.vue'
import router from './router'
import {createPinia} from 'pinia'
import {i18n} from './i18n'
import './assets/styles/tokens.css'
import './assets/styles/themes.css'
import './assets/styles/main.css'

createApp(App).use(createPinia()).use(router).use(i18n).mount('#app')
