window.addEventListener('DOMContentLoaded', () => {
    const main = document.getElementsByTagName('main')[0];
    if (main.attributes.getNamedItem('data-vote-enabled').value === 'true') {
        for (const div of document.getElementsByClassName('answer-row')) {
            console.log(div)
            div.addEventListener('click', () => {
                div.submit()
            });
        }
    }
})