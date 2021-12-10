/*
**  Pure-UUID -- Pure JavaScript Based Universally Unique Identifier (UUID)
**  Copyright (c) 2004-2021 Dr. Ralf S. Engelschall <rse@engelschall.com>
**
**  Permission is hereby granted, free of charge, to any person obtaining
**  a copy of this software and associated documentation files (the
**  "Software"), to deal in the Software without restriction, including
**  without limitation the rights to use, copy, modify, merge, publish,
**  distribute, sublicense, and/or sell copies of the Software, and to
**  permit persons to whom the Software is furnished to do so, subject to
**  the following conditions:
**
**  The above copyright notice and this permission notice shall be included
**  in all copies or substantial portions of the Software.
**
**  THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
**  EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
**  MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
**  IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
**  CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
**  TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
**  SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

/* jshint -W030: false */
/* eslint no-unused-expressions: 0 */

global.chai = require("chai");
global.expect = global.chai.expect;
global.chai.config.includeStack = true;

var UUID = require("./uuid.js");

describe("UUID base functionality", function () {
    it("should provide basic object functionality", function () {
        var uuid = new UUID();
        expect(uuid).to.be.a("object");
        expect(uuid).to.respondTo("make");
        expect(uuid).to.respondTo("parse");
        expect(uuid).to.respondTo("format");
    });
    it("should parse and format standard UUIDs", function () {
        expect(new UUID().format())
            .to.be.equal("00000000-0000-0000-0000-000000000000");
        expect(new UUID().parse("nil").format())
            .to.be.equal("00000000-0000-0000-0000-000000000000");
        expect(new UUID().parse("ns:DNS").format())
            .to.be.equal("6ba7b810-9dad-11d1-80b4-00c04fd430c8");
        expect(new UUID().parse("ns:OID").format())
            .to.be.equal("6ba7b812-9dad-11d1-80b4-00c04fd430c8");
        expect(new UUID().parse("ns:X500").format())
            .to.be.equal("6ba7b814-9dad-11d1-80b4-00c04fd430c8");
        expect(new UUID("7da78284-2f14-5e7f-95e1-baaa9027c26f").format())
            .to.be.equal("7da78284-2f14-5e7f-95e1-baaa9027c26f");
        expect(new UUID().parse("7da78284-2f14-5e7f-95e1-baaa9027c26f").format())
            .to.be.equal("7da78284-2f14-5e7f-95e1-baaa9027c26f");
        expect(new UUID().parse("7da78284-2f14-5e7f-95e1-baaa9027c26f").export())
            .to.be.deep.equal([0x7d,0xa7,0x82,0x84,0x2f,0x14,0x5e,0x7f,0x95,0xe1,0xba,0xaa,0x90,0x27,0xc2,0x6f]);
        expect(new UUID().import([0x7d,0xa7,0x82,0x84,0x2f,0x14,0x5e,0x7f,0x95,0xe1,0xba,0xaa,0x90,0x27,0xc2,0x6f]).format())
            .to.be.equal("7da78284-2f14-5e7f-95e1-baaa9027c26f");
        expect(new UUID().import([0x7d,0xa7,0x82,0x84,0x2f,0x14,0x5e,0x7f,0x95,0xe1,0xba,0xaa,0x90,0x27,0xc2,0x6f]).format("b16"))
            .to.be.equal("7DA782842F145E7F95E1BAAA9027C26F");
        expect(new UUID().parse("7DA782842F145E7F95E1BAAA9027C26F", "b16").export())
            .to.be.deep.equal([0x7d,0xa7,0x82,0x84,0x2f,0x14,0x5e,0x7f,0x95,0xe1,0xba,0xaa,0x90,0x27,0xc2,0x6f]);
        expect(new UUID().import([0x7d,0xa7,0x82,0x84,0x2f,0x14,0x5e,0x7f,0x95,0xe1,0xba,0xaa,0x90,0x27,0xc2,0x6f]).format("z85"))
            .to.be.equal("Ew.WIfbd-xMePrOKsd[-");
        expect(new UUID().parse("Ew.WIfbd-xMePrOKsd[-", "z85").export())
            .to.be.deep.equal([0x7d,0xa7,0x82,0x84,0x2f,0x14,0x5e,0x7f,0x95,0xe1,0xba,0xaa,0x90,0x27,0xc2,0x6f]);
        expect(JSON.stringify(new UUID("7da78284-2f14-5e7f-95e1-baaa9027c26f")))
            .to.be.equal("\"7da78284-2f14-5e7f-95e1-baaa9027c26f\"");
    });
    it("should be able to make various UUID versions", function () {
        expect(new UUID(1).format())
            .to.be.not.empty;
        expect(new UUID(3, "ns:URL", "foo").format())
            .to.be.equal("a5bf60bd-fe2d-3fac-bbd7-404751e6ca66");
        expect(new UUID(4).format())
            .to.be.not.empty;
        expect(new UUID(5, "ns:URL", "foo").format())
            .to.be.equal("7da78284-2f14-5e7f-95e1-baaa9027c26f");
        expect(new UUID(5, new UUID("6ba7b811-9dad-11d1-80b4-00c04fd430c8"), "foo").format())
            .to.be.equal("7da78284-2f14-5e7f-95e1-baaa9027c26f");
    });
    it("should be able to fold UUIDs", function () {
        expect(new UUID("nil").fold(4))
            .to.be.deep.equal([ 0x00 ]);
        expect(new UUID("nil").fold(3))
            .to.be.deep.equal([ 0x00, 0x00 ]);
        expect(new UUID("nil").fold(2))
            .to.be.deep.equal([ 0x00, 0x00, 0x00, 0x00 ]);
        expect(new UUID("nil").fold(1))
            .to.be.deep.equal([ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00 ]);
        expect(new UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8").fold(4))
            .to.be.deep.equal([ 0x03 ]);
        expect(new UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8").fold(3))
            .to.be.deep.equal([ 0xa0, 0xa3 ]);
        expect(new UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8").fold(2))
            .to.be.deep.equal([ 0x39, 0x6a, 0x99, 0xc9 ]);
        expect(new UUID("6ba7b810-9dad-11d1-80b4-00c04fd430c8").fold(1))
            .to.be.deep.equal([ 0xeb, 0x13, 0xb8, 0xd0, 0xd2, 0x79, 0x21, 0x19 ]);
    });
    it("should be able to detect errors", function () {
        expect(function () { new UUID().parse("00000000-0000-0000-0000-000000000000"); })
            .to.not.throw(Error);
        expect(function () { new UUID().parse("xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx"); })
            .to.throw(Error);
    });
});

