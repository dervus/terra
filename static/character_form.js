document.addEventListener("DOMContentLoaded", (event) => {
    gender = Kefir.fromEvents($('input[name="gender"]'), "input", e => e.target).filter(e => e.checked).map(e => e.value).toProperty(() => "male")
    race = Kefir.fromEvents($('input[name="race"]'), "input", e => e.target).filter(e => e.checked).map(e => e.value).toProperty(() => "human")
    klass = Kefir.fromEvents($('input[name="class"]'), "input", e => e.target).filter(e => e.checked).map(e => e.value).toProperty(() => "warrior")
    form = Kefir.zip([gender, race, klass], (g, r, c) => { return {gender: g, race: r, klass: c} }).toProperty()
    
    let traits = document.querySelectorAll('input:checked[name="trait"]').length

    for (let node of document.querySelectorAll('input[name="trait"] + label, input[name="location"] + label').values()) {
        node.addEventListener("mouseenter", (event) => {
            let target = document.getElementById(event.target.htmlFor)
            let id = target.value
            let kind = target.name

            for (let descNode of document.querySelectorAll(`.description[data-kind="${kind}"]`)) {
                descNode.classList.add("hidden")
            }
            
            let descNode = document.querySelector(`.description[data-kind="${kind}"][data-entity="${id}"]`)
            if (descNode) descNode.classList.remove("hidden")
        })
    }
    for (let node of document.querySelectorAll('input[name="trait"]').values()) {
        node.addEventListener("change", (event) => {
            if (event.target.checked) traits += 1
            else traits -= 1

            document.querySelectorAll('input[name="trait"]').forEach(el => {
                if (!el.checked) el.disabled = (traits >= 2)
            })
        })
    }
})
