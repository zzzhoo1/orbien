import {createApp} from 'vue'
import App from './App.vue'
import {router} from './router'
import {createPinia} from 'pinia'
import {i18n} from './i18n'
import './assets/styles/main.css'

createApp(App).use(createPinia()).use(router).use(i18n).mount('#app')
