// Biospheres Toolbox Definition
// Consolidated toolbox for Biospheres mode
// Requirements: 1.3, 4.4, 9.1, 9.2, 10.5

const BiospheresToolbox = {
    mode: "biospheres",
    displayName: "Biospheres",
    color: "#00BCD4",
    
    // Toolbox structure
    getToolbox: function() {
        return {
            kind: "categoryToolbox",
            contents: [
                {
                    kind: "category",
                    name: "Blocks",
                    colour: "#00BCD4",
                    contents: [
                        {
                            kind: "label",
                            text: "Toolbox cleared - ready to repopulate"
                        }
                    ]
                }
            ]
        };
    }
};
