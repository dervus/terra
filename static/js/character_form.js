// let ENTITY_KINDS = ['race', 'model', 'class', 'armorset', 'weaponset', 'traits', 'location']
// let INFO = {}
// let INFO_BUILDERS = {}

// function getTraitsConfig() {
//     let dataset = document.getElementById('character-form').dataset
//     return {
//         limit: Number(dataset.maxTraits),
//         maxCost: Number(dataset.maxTraitsCost)
//     }
// }

// function entityFilter(src) {
//     let [type, ...items] = src.split(' ')

//     if (type === 'pass') {
//         return (_value => true)
//     } else if (type === 'allow:') {
//         return (value => items.includes(value))
//     } else if (type === 'deny:') {
//         return (value => !items.includes(value))
//     } else {
//         throw `invalid entity filter: ${type}`
//     }
// }

// function genderFilter(src) {
//     if (src === 'any') {
//         return (_value => true)
//     } else if (src === 'male-only') {
//         return (value => value === 'male')
//     } else if (src === 'female-only') {
//         return (value => value === 'female')
//     } else {
//         throw `invalid gender filter: ${src}`
//     }
// }

// function armorSkillFilter(src) {
//     let threshold = Number(src)
//     return (value => threshold <= Number(value))
// }

// function weaponSkillFilter(src) {
//     let requiredSkills = src.split(' ')
//     return (value => {
//         let availableSkills
//         if (Array.isArray(value)) availableSkills = value
//         else availableSkills = value.split(' ')

//         return requiredSkills.every(skill => availableSkills.includes(skill))
//     })
// }

// INFO_BUILDERS.race = function getRaceInfo(input) {
//     return {
//         id: input.value
//     }
// }

// INFO_BUILDERS.model = function getModelInfo(input) {
//     return {
//         id: input.value,
//         allowsRace: entityFilter(input.dataset.raceFilter),
//         allowsGender: genderFilter(input.dataset.genderFilter)
//     }
// }

// INFO_BUILDERS.class = function getClassInfo(input) {
//     return {
//         id: input.value,
//         allowsRace: entityFilter(input.dataset.raceFilter),
//         allowsGender: genderFilter(input.dataset.genderFilter),
//         armorSkill: Number(input.dataset.armorSkill),
//         weaponSkills: input.dataset.weaponSkills.split(' ')
//     }
// }

// INFO_BUILDERS.armorset = function getArmorSetInfo(input) {
//     return {
//         id: input.value,
//         allowsRace: entityFilter(input.dataset.raceFilter),
//         allowsGender: genderFilter(input.dataset.genderFilter),
//         allowsClass: entityFilter(input.dataset.classFilter),
//         allowsArmorSkill: armorSkillFilter(input.dataset.armorSkill)
//     }
// }

// INFO_BUILDERS.weaponset = function getWeaponSetInfo(input) {
//     return {
//         id: input.value,
//         allowsRace: entityFilter(input.dataset.raceFilter),
//         allowsGender: genderFilter(input.dataset.genderFilter),
//         allowsClass: entityFilter(input.dataset.classFilter),
//         allowsWeaponSkills: weaponSkillFilter(input.dataset.weaponSkills)
//     }
// }

// INFO_BUILDERS.traits = function getTraitInfo(input) {
//     return {
//         id: input.value,
//         cost: Number(input.dataset.cost),
//         group: input.dataset.group,
//         unique: Boolean(input.dataset.unique),
//         allowsRace: entityFilter(input.dataset.raceFilter),
//         allowsGender: genderFilter(input.dataset.genderFilter),
//         allowsClass: entityFilter(input.dataset.classFilter)
//     }
// }

// INFO_BUILDERS.location = function getLocationInfo(input) {
//     return {
//         id: input.value,
//         allowsRace: entityFilter(input.dataset.raceFilter),
//         allowsGender: genderFilter(input.dataset.genderFilter),
//         allowsClass: entityFilter(input.dataset.classFilter)
//     }
// }

// function buildInfoFor(kind) {
//     let store = INFO[kind] = {}
//     for (let el of document.querySelectorAll(`#character-form input[name="${kind}"]`).values()) {
//         store[el.value] = INFO_BUILDERS[kind](el)
//     }
// }

// function getFormData() {
//     let data = new FormData(document.getElementById('character-form'))

//     let race = data.get('race')
//     let gender = data.get('gender')
//     let model = data.get('model')
//     let klass = data.get('class')
//     let armorSkill = klass ? INFO.class[klass].armorSkill : null
//     let weaponSkills = klass ? INFO.class[klass].weaponSkills : null
    
//     return {
//         race,
//         gender,
//         model,
//         class: klass,
//         armorSkill,
//         weaponSkills
//     }
// }

// function updateVisability(kind) {
//     let form = getFormData()
//     for (let el of document.querySelectorAll(`#character-form input[name="${kind}"]`).values()) {
//         let info = INFO[el.name][el.value]
//         let result = true;

//         let check = (filter, value) => {
//             if (result && filter) {
//                 result = filter(value)
//             }
//         }

//         check(info.allowsRace, form.race)
//         check(info.allowsGender, form.gender)
//         check(info.allowsClass, form.class)
//         check(info.allowsArmorSkill, form.armorSkill)
//         check(info.allowsWeaponSkills, form.weaponSkills)

//         if (result) {
//             el.disabled = false
//             el.parentElement.classList.remove('hidden')
//         } else if (el.name == 'class') {
//             el.checked = false
//             el.disabled = true
//         } else {
//             el.checked = false
//             el.disabled = true
//             el.parentElement.classList.add('hidden')
//         }
//     }
//     if (form.race && form.class && form.gender) {
//         for (let el of document.querySelectorAll('.second-part')) {
//             el.classList.add('active')
//         }
//     }
// }

function init() {
    // let traitsCfg = getTraitsConfig()
    // ENTITY_KINDS.forEach(buildInfoFor)
    
    let traits = document.querySelectorAll('input:checked[name="traits"]').length

    for (let node of document.querySelectorAll('input + label').values()) {
        node.addEventListener('mouseenter', (event) => {
            let target = document.getElementById(event.target.htmlFor)
            let id = target.value
            let kind = target.name

            for (let descNode of document.querySelectorAll('.entity-info')) {
                descNode.classList.add('hidden')
            }
            
            let descNode = document.querySelector(`.entity-info[data-entity="${id}"]`)
            if (descNode) descNode.classList.remove('hidden')
        })
        node.addEventListener('mouseleave', (event) => {
            let id = document.getElementById(event.target.htmlFor).value
            let descNode = document.querySelector(`.entity-info[data-entity="${id}"]`)
            if (descNode) descNode.classList.add('hidden')
        })
    }

    for (let node of document.querySelectorAll('input[name="traits"]').values()) {
        node.addEventListener('change', (event) => {
            if (event.target.checked) traits += 1
            else traits -= 1

            document.querySelectorAll('input[name="traits"]').forEach(el => {
                if (!el.checked) el.disabled = (traits >= 2)
            })
        })
    }

    // for (let el of document.querySelectorAll('#character-form input[type="radio"], #character-form input[type="checkbox"]').values()) {
    //     el.addEventListener('change', event => {
    //         ENTITY_KINDS.forEach(updateVisability)
    //     })
    // }
}

document.addEventListener('DOMContentLoaded', init)
// window.addEventListener("beforeunload", (event) => event.preventDefault())
