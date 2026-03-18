const NAMES = [
	// Male
	'Abe', 'Ace', 'Ajax', 'Alec', 'Arlo', 'Ash', 'Axel',
	'Beck', 'Benny', 'Blake', 'Bo', 'Bram', 'Brett', 'Brice',
	'Cal', 'Cane', 'Carl', 'Cash', 'Cato', 'Clay', 'Clem', 'Cole', 'Cruz',
	'Dale', 'Dash', 'Dave', 'Dean', 'Dell', 'Dirk', 'Drake', 'Drew',
	'Earl', 'Eli', 'Emre', 'Erik',
	'Finn', 'Flint', 'Ford', 'Fox', 'Frank',
	'Gage', 'Gale', 'Gene', 'Gus',
	'Hal', 'Hank', 'Hawk', 'Hugh', 'Hugo',
	'Ivan',
	'Jack', 'Jake', 'Jax', 'Joel', 'John', 'Jules',
	'Kane', 'Karl', 'Kent', 'Knox', 'Kurt',
	'Lars', 'Levi', 'Liam', 'Link', 'Luca', 'Luke',
	'Marc', 'Marsh', 'Matt', 'Max', 'Miles', 'Milo',
	'Nash', 'Neal', 'Neil', 'Nick', 'Noel',
	'Oak', 'Omar', 'Otis', 'Owen',
	'Paul', 'Penn', 'Pete', 'Pierce',
	'Quinn',
	'Rand', 'Ray', 'Reed', 'Reid', 'Rex', 'Rhett', 'Rio', 'Robb', 'Rory', 'Ross', 'Roy', 'Russ',
	'Sam', 'Scout', 'Sean', 'Seth', 'Slate', 'Soren', 'Sven',
	'Tate', 'Theo', 'Thorn', 'Todd', 'Tom', 'Troy',
	'Umar',
	'Vance', 'Vinn',
	'Wade', 'Walt', 'Ward', 'Wolf',
	'Xan',
	'Yuri',
	'Zach', 'Zane', 'Zed', 'Zeus',
	// Female
	'Ada', 'Aida', 'Alma', 'Alta', 'Amber', 'Amy', 'Arden', 'Aria', 'Arya',
	'Bea', 'Blair', 'Bonnie', 'Bree', 'Brook',
	'Cara', 'Carmen', 'Cass', 'Cleo', 'Cora',
	'Dana', 'Dawn', 'Dell', 'Demi', 'Drew',
	'Eden', 'Effie', 'Elise', 'Ella', 'Ember', 'Eve',
	'Fawn', 'Faye', 'Fern', 'Flora',
	'Gail', 'Gem', 'Gwen',
	'Hana', 'Harlow', 'Haven', 'Hope', 'Hunter',
	'Ida', 'Imara', 'Ines', 'Iris', 'Ivy',
	'Jade', 'Jane', 'Jess', 'Joan', 'Joy', 'June',
	'Kai', 'Kaia', 'Kate', 'Kay', 'Keiko', 'Kim',
	'Lana', 'Lark', 'Lea', 'Lena', 'Lexi', 'Lily', 'Lola', 'Lora', 'Luna',
	'Mae', 'Mara', 'Maya', 'Mira', 'Mona',
	'Nadia', 'Nina', 'Nola', 'Nora', 'Nova',
	'Opal',
	'Page', 'Pearl', 'Petra', 'Pia',
	'Reeve', 'Remi', 'Ren', 'Rhea', 'Rosa', 'Rose', 'Ruby', 'Ruth',
	'Sage', 'Sara', 'Seren', 'Skye', 'Sloane', 'Sol',
	'Tara', 'Tess', 'Thea',
	'Uma',
	'Val', 'Vera', 'Viv',
	'Willa', 'Wren',
	'Xena',
	'Yara', 'Yuki',
	'Zara', 'Zoe',
];

/**
 * Pick a random session name that isn't already in use.
 * @param usedNames - set of names currently in use across sessions
 */
export function pickSessionName(usedNames: Set<string>): string {
	const available = NAMES.filter(n => !usedNames.has(n));
	const pool = available.length > 0 ? available : NAMES; // fallback if all taken
	return pool[Math.floor(Math.random() * pool.length)];
}
