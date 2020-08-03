package writes.ids

import org.scalatest.{FlatSpec, Matchers}
import util._

class AutoIncrementCreateSpec extends FlatSpec with Matchers with ApiSpecBase {

  //Mysql only one, @unique or @@index
  //postgres unlimited
  //sqlite only id
  "Creating an item with a non primary key autoincrement" should "work" in {
    val project = ProjectDsl.fromString {
      s"""
         |model Mail {
         |    id Int   @default(autoincrement())
         |    messageId Int @id
         |
         |    @@index(id)
         |}
       """.stripMargin
    }
    database.setup(project)

    val result = server.query(
      """
        |mutation {
        |  createMail(data: { messageId:1 }){
        |    id
        |    messageId
        |  }
        |}
      """.stripMargin,
      project
    )

    result.toString() should be("{\"data\":{\"createMail\":{\"id\":1,\"messageId\":1}}}")

  }
}
