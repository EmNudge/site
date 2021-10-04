import { getDefferedPromise } from "../../utils/getDefferedPromise";

const getTimestamps = async (baseUrl: string) => {
    const timestampsText = await (await fetch(baseUrl + '.timestamps')).text();
    return timestampsText.split(/\s+/).map(time => {
        const [minutes, seconds] = time.split(':');
        return Number(minutes) * 60 + Number(seconds);
    });
}

const getAudioSource = (url: string, type: string) => {
    const source = document.createElement('source');
    source.setAttribute('src', url);
    source.setAttribute('type', 'audio/' + type);

    return source;
}

const getAudio = (baseUrl: string) => {
    const audio = document.createElement('audio');
    audio.appendChild(getAudioSource(baseUrl + '.ogg', 'ogg'));
    audio.appendChild(getAudioSource(baseUrl + '.mp3', 'mpeg'));

    const [promise, res] = getDefferedPromise<HTMLAudioElement>();

    audio.addEventListener('canplaythrough', () => res(audio));

    return promise;
}

export const getDataForRecording = async (recording: string) => {
    const baseUrl = `/recordings/${recording}`;
    const [timestamps, audio] = await Promise.all([getTimestamps(baseUrl), getAudio(baseUrl)])

    return { timestamps, audio }
}

export const getParagraphIndex = (currentTime: number, timestamps: number[]) => {
    let i = 0;
    while (i < timestamps.length && currentTime > timestamps[i + 1]) i++;

    return i;
}

export const updateParagraphHighlight = (paragraphs: HTMLParagraphElement[], oldIndex: number, newIndex: number) => {
    if (oldIndex >= 0) paragraphs[oldIndex].classList.remove('speaking');
    if (newIndex >= 0) {
        paragraphs[newIndex].classList.add('speaking');
        paragraphs[newIndex].scrollIntoView({
            behavior: 'smooth',
            block: 'center',
            inline: 'nearest'
        });
    }
}

export const setAudio = (audio: HTMLAudioElement, isPlaying: boolean) => {
    if (isPlaying) audio.play();
    else audio.pause();
}


export const getParagraphHighlighter = (paragraphs: HTMLParagraphElement[]) => {
    let index = -1;

    return {
        clear() {
            updateParagraphHighlight(paragraphs, index, -1);
            index = -1;
        },
        highlight(newIndex: number) {
            if (newIndex === index) return;

            updateParagraphHighlight(paragraphs, index, newIndex);
            index = newIndex;
        },
    }
}