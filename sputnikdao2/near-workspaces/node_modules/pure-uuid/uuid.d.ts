/*!
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

interface UUID {
    /*  making  */
    make(version: number, ...params: any[]): UUID;

    /*  parsing  */
    parse(str: string): UUID;

    /*  formatting  */
    format(type?: string): string;

    /*  formatting (alias)  */
    toString(type?: string): string;

    /*  sensible JSON serialization  */
    toJSON(): string;

    /*  importing  */
    import(arr: number[]): UUID;

    /*  exporting  */
    export(): number[];

    /*  byte-wise comparison  */
    compare(other: UUID): number;

    /*  equal check  */
    equal(other: UUID): boolean;

    /*  fold 1-4 times  */
    fold(k: number): number[];
}

export interface UUIDConstructor {
  /*  default construction  */
  new(): UUID;

  /*  parsing construction  */
  new(uuid: string): UUID;

  /*  making construction  */
  new(version: number): UUID;
  new(version: number, ns: string, data: string): UUID;
}

declare var UUID: UUIDConstructor;
export default UUID;

