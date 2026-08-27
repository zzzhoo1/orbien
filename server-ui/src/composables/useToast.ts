import {computed, readonly, ref} from 'vue'

export type ToastType = 'info' | 'error'

export interface ToastMessage {
  id: number
  type: ToastType
  text: string
}

const current = ref<ToastMessage | null>(null)
let timer: number | null = null
let nextId = 1

function show(type: ToastType, text: string, duration = 3000) {
  current.value = {id: nextId++, type, text}
  if (timer !== null) {
    window.clearTimeout(timer)
  }
  timer = window.setTimeout(() => {
    current.value = null
    timer = null
  }, duration)
}

export function useToast() {
  const message = computed(() => current.value)

  return {
    message: readonly(message),
    show,
  }
}
